//! Sessions table widget - displays Claude Code session history

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use super::tabs::{Tab, TabBar};
use crate::services::session_metadata::extract_issue_id;
use crate::tui::theme::Theme;
use crate::types::SessionInfo;

/// Maximum content width (consistent with other views)
const MAX_CONTENT_WIDTH: u16 = 170;

/// Column indices
const COL_PROJECT: usize = 0;
const COL_ISSUE: usize = 1;
const COL_SUMMARY: usize = 2;
const COL_BRANCH: usize = 3;
const COL_DATE: usize = 4;
const COL_DURATION: usize = 5;
const COL_COST: usize = 6;

/// Column definitions: (label, width)
const COLUMNS: [(&str, u16); 7] = [
    ("Project", 16),  // 0: COL_PROJECT (14 + 2 marker)
    ("Issue", 12),    // 1: COL_ISSUE
    ("Summary", 40),  // 2: COL_SUMMARY
    ("Branch", 18),   // 3: COL_BRANCH
    ("Date", 18),     // 4: COL_DATE
    ("Duration", 10), // 5: COL_DURATION
    ("Cost", 10),     // 6: COL_COST
];

/// Determine which columns are visible for a given terminal width.
/// Columns are hidden in priority order: Branch first, then Duration, then Issue.
fn visible_columns(width: u16) -> Vec<usize> {
    const HIDE_ORDER: [usize; 3] = [COL_BRANCH, COL_DURATION, COL_ISSUE];

    let mut visible: Vec<usize> = (0..COLUMNS.len()).collect();

    for &col_idx in &HIDE_ORDER {
        let total: u16 = visible.iter().map(|&i| COLUMNS[i].1).sum();
        if total <= width {
            return visible;
        }
        visible.retain(|&i| i != col_idx);
    }

    visible
}

/// Calculate total table width for visible columns
fn table_width_for(visible: &[usize]) -> u16 {
    visible.iter().map(|&i| COLUMNS[i].1).sum()
}

/// Sort mode for the sessions table
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SessionSort {
    #[default]
    DateDesc,
    DateAsc,
    CostDesc,
    CostAsc,
}

impl SessionSort {
    /// Cycle to the next sort mode
    pub fn next(self) -> Self {
        match self {
            Self::DateDesc => Self::DateAsc,
            Self::DateAsc => Self::CostDesc,
            Self::CostDesc => Self::CostAsc,
            Self::CostAsc => Self::DateDesc,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::DateDesc => "Date ↓",
            Self::DateAsc => "Date ↑",
            Self::CostDesc => "Cost ↓",
            Self::CostAsc => "Cost ↑",
        }
    }

    /// Sort a slice of sessions in place
    pub fn sort(self, sessions: &mut [SessionInfo]) {
        match self {
            Self::DateDesc => sessions.sort_by(|a, b| b.created.cmp(&a.created)),
            Self::DateAsc => sessions.sort_by(|a, b| a.created.cmp(&b.created)),
            Self::CostDesc => sessions.sort_by(|a, b| {
                b.total_cost_usd
                    .partial_cmp(&a.total_cost_usd)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            Self::CostAsc => sessions.sort_by(|a, b| {
                a.total_cost_usd
                    .partial_cmp(&b.total_cost_usd)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
        }
    }
}

/// Sessions table view
pub struct SessionsView<'a> {
    sessions: &'a [SessionInfo],
    scroll_offset: usize,
    selected_index: Option<usize>,
    selected_tab: Tab,
    sort: SessionSort,
    theme: Theme,
}

impl<'a> SessionsView<'a> {
    pub fn new(
        sessions: &'a [SessionInfo],
        scroll_offset: usize,
        selected_index: Option<usize>,
        selected_tab: Tab,
        sort: SessionSort,
        theme: Theme,
    ) -> Self {
        Self {
            sessions,
            scroll_offset,
            selected_index,
            selected_tab,
            sort,
            theme,
        }
    }

    /// Calculate max scroll offset
    #[allow(dead_code)] // Used in tests
    pub fn max_scroll_offset(count: usize, visible_rows: usize) -> usize {
        count.saturating_sub(visible_rows)
    }
}

impl Widget for SessionsView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content_width = area.width.min(MAX_CONTENT_WIDTH);
        let x_offset = (area.width.saturating_sub(content_width)) / 2;
        let centered_area = Rect {
            x: area.x + x_offset,
            y: area.y,
            width: content_width,
            height: area.height,
        };

        let chunks = Layout::vertical([
            Constraint::Length(1), // 0: Top padding
            Constraint::Length(1), // 1: Tab bar
            Constraint::Length(1), // 2: Separator
            Constraint::Length(1), // 3: Header
            Constraint::Fill(1),   // 4: Session rows (fill remaining)
            Constraint::Length(1), // 5: Separator
            Constraint::Length(1), // 6: Keybindings
        ])
        .split(centered_area);

        // Render tab bar
        TabBar::new(self.selected_tab, self.theme).render(chunks[1], buf);

        // Render separator
        render_separator(chunks[2], buf, self.theme);

        let visible = visible_columns(centered_area.width);

        // Render header
        self.render_header(chunks[3], buf, &visible);

        // Render session rows
        self.render_rows(chunks[4], buf, &visible);

        // Render separator
        render_separator(chunks[5], buf, self.theme);

        // Render keybindings
        self.render_keybindings(chunks[6], buf);
    }
}

impl SessionsView<'_> {
    fn render_header(&self, area: Rect, buf: &mut Buffer, visible: &[usize]) {
        let tw = table_width_for(visible);
        let offset = area.width.saturating_sub(tw) / 2;
        let header_style = Style::default()
            .fg(self.theme.text())
            .add_modifier(Modifier::BOLD);

        let mut spans = Vec::new();
        for &col in visible {
            let (label, width) = COLUMNS[col];
            // Add sort arrow to the active sort column
            let label_with_arrow = match (col, self.sort) {
                (COL_DATE, SessionSort::DateDesc) => format!("{} ↓", label),
                (COL_DATE, SessionSort::DateAsc) => format!("{} ↑", label),
                (COL_COST, SessionSort::CostDesc) => format!("{} ↓", label),
                (COL_COST, SessionSort::CostAsc) => format!("{} ↑", label),
                _ => label.to_string(),
            };
            let formatted = if col == COL_PROJECT {
                format!(
                    "  {:<width$}",
                    label_with_arrow,
                    width = (width as usize) - 2
                )
            } else if col == COL_SUMMARY || col == COL_BRANCH || col == COL_DATE || col == COL_ISSUE
            {
                format!("{:<width$}", label_with_arrow, width = width as usize)
            } else {
                format!("{:>width$}", label_with_arrow, width = width as usize)
            };
            spans.push(Span::styled(formatted, header_style));
        }

        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Left)
            .render(
                Rect {
                    x: area.x + offset,
                    y: area.y,
                    width: tw.min(area.width),
                    height: area.height,
                },
                buf,
            );
    }

    fn render_rows(&self, area: Rect, buf: &mut Buffer, visible: &[usize]) {
        let tw = table_width_for(visible);
        let offset = area.width.saturating_sub(tw) / 2;
        let start = self.scroll_offset;

        // Each row takes 1 line, but selected row takes 2 (+ first prompt)
        let mut y = area.y;
        let mut idx = start;

        while y < area.y + area.height && idx < self.sessions.len() {
            let session = &self.sessions[idx];
            let is_selected = self.selected_index == Some(idx);

            let row_area = Rect {
                x: area.x + offset,
                y,
                width: tw.min(area.width),
                height: 1,
            };

            self.render_row(row_area, buf, session, visible, is_selected);
            y += 1;

            // Show first prompt as secondary line when selected
            if is_selected && y < area.y + area.height {
                let prompt_area = Rect {
                    x: area.x + offset,
                    y,
                    width: tw.min(area.width),
                    height: 1,
                };
                self.render_first_prompt(prompt_area, buf, session);
                y += 1;
            }

            idx += 1;
        }
    }

    fn render_row(
        &self,
        area: Rect,
        buf: &mut Buffer,
        session: &SessionInfo,
        visible: &[usize],
        is_selected: bool,
    ) {
        use chrono::Local;

        let selection_modifier = if is_selected {
            Modifier::BOLD | Modifier::REVERSED
        } else {
            Modifier::empty()
        };

        let mut spans = Vec::new();

        for (col_idx, &col) in visible.iter().enumerate() {
            let (text, base_style) = match col {
                COL_PROJECT => {
                    let marker = if is_selected { "▸ " } else { "  " };
                    let name = truncate_str(&session.project, 14);
                    (
                        format!("{}{:<14}", marker, name),
                        Style::default().fg(self.theme.accent()),
                    )
                }
                COL_ISSUE => {
                    let issue = session
                        .metadata
                        .as_ref()
                        .and_then(|m| m.issue_id.clone())
                        .or_else(|| extract_issue_id(&session.git_branch))
                        .unwrap_or_else(|| "—".to_string());
                    let issue = truncate_str(&issue, 12);
                    (
                        format!("{:<12}", issue),
                        Style::default().fg(self.theme.accent()),
                    )
                }
                COL_SUMMARY => {
                    let text = if session.summary.is_empty() {
                        &session.first_prompt
                    } else {
                        &session.summary
                    };
                    let summary = truncate_str(text, 40);
                    (
                        format!("{:<40}", summary),
                        Style::default().fg(self.theme.text()),
                    )
                }
                COL_BRANCH => {
                    let branch = if session.git_branch.is_empty() {
                        "—"
                    } else {
                        &session.git_branch
                    };
                    let branch = truncate_str(branch, 18);
                    (
                        format!("{:<18}", branch),
                        Style::default().fg(self.theme.date()),
                    )
                }
                COL_DATE => {
                    let local = session.created.with_timezone(&Local);
                    let date_str = local.format("%b %d, %l:%M %p").to_string();
                    let date_str = truncate_str(&date_str, 18);
                    (
                        format!("{:<18}", date_str),
                        Style::default().fg(self.theme.date()),
                    )
                }
                COL_DURATION => {
                    let duration = format_duration(session.modified, session.created);
                    (
                        format!("{:>10}", duration),
                        Style::default().fg(self.theme.text()),
                    )
                }
                COL_COST => (
                    format!("{:>10}", format!("${:.2}", session.total_cost_usd)),
                    Style::default().fg(self.theme.cost()),
                ),
                _ => unreachable!(),
            };

            let style = if is_selected && col_idx > 0 {
                base_style.add_modifier(selection_modifier)
            } else if is_selected && col_idx == 0 {
                base_style.add_modifier(Modifier::BOLD)
            } else {
                base_style
            };

            spans.push(Span::styled(text, style));
        }

        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Left)
            .render(area, buf);
    }

    fn render_first_prompt(&self, area: Rect, buf: &mut Buffer, session: &SessionInfo) {
        let prompt = if session.first_prompt.is_empty() {
            "(no prompt)"
        } else {
            &session.first_prompt
        };
        let max_len = area.width.saturating_sub(4) as usize;
        let truncated = truncate_str(prompt, max_len);

        Paragraph::new(Line::from(vec![
            Span::raw("    "),
            Span::styled(truncated, Style::default().fg(self.theme.muted())),
        ]))
        .alignment(Alignment::Left)
        .render(area, buf);
    }

    fn render_keybindings(&self, area: Rect, buf: &mut Buffer) {
        let sort_label = format!(": Sort ({})", self.sort.label());
        let bindings = Paragraph::new(Line::from(vec![
            Span::styled("↑↓", Style::default().fg(self.theme.accent())),
            Span::styled(": Navigate", Style::default().fg(self.theme.muted())),
            Span::raw("  "),
            Span::styled("s", Style::default().fg(self.theme.accent())),
            Span::styled(sort_label, Style::default().fg(self.theme.muted())),
            Span::raw("  "),
            Span::styled("Enter", Style::default().fg(self.theme.accent())),
            Span::styled(": Details", Style::default().fg(self.theme.muted())),
            Span::raw("  "),
            Span::styled("Tab", Style::default().fg(self.theme.accent())),
            Span::styled(": Switch view", Style::default().fg(self.theme.muted())),
            Span::raw("  "),
            Span::styled("?", Style::default().fg(self.theme.accent())),
            Span::styled(": Help", Style::default().fg(self.theme.muted())),
        ]))
        .alignment(Alignment::Center);

        bindings.render(area, buf);
    }
}

/// Format a duration between two timestamps as human-readable
fn format_duration(
    modified: chrono::DateTime<chrono::Utc>,
    created: chrono::DateTime<chrono::Utc>,
) -> String {
    let secs = (modified - created).num_seconds().max(0);
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;

    if hours > 0 {
        format!("{}h {:02}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

/// Truncate a string to max chars, appending "…" if truncated
fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        format!(
            "{}…",
            s.chars()
                .take(max_chars.saturating_sub(1))
                .collect::<String>()
        )
    }
}

fn render_separator(area: Rect, buf: &mut Buffer, theme: Theme) {
    let line = "─".repeat(area.width as usize);
    buf.set_string(area.x, area.y, &line, Style::default().fg(theme.muted()));
}

/// Compute visible rows for sessions tab (same overhead as dashboard)
pub fn sessions_visible_rows(terminal_height: u16) -> usize {
    // padding(1) + tabs(1) + sep(1) + header(1) + sep(1) + keybindings(1) = 6
    terminal_height.saturating_sub(6) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_hours_and_minutes() {
        use chrono::{TimeZone, Utc};
        let created = Utc.with_ymd_and_hms(2026, 1, 1, 10, 0, 0).unwrap();
        let modified = Utc.with_ymd_and_hms(2026, 1, 1, 11, 23, 0).unwrap();
        assert_eq!(format_duration(modified, created), "1h 23m");
    }

    #[test]
    fn test_format_duration_minutes_only() {
        use chrono::{TimeZone, Utc};
        let created = Utc.with_ymd_and_hms(2026, 1, 1, 10, 0, 0).unwrap();
        let modified = Utc.with_ymd_and_hms(2026, 1, 1, 10, 42, 0).unwrap();
        assert_eq!(format_duration(modified, created), "42m");
    }

    #[test]
    fn test_format_duration_zero() {
        use chrono::{TimeZone, Utc};
        let ts = Utc.with_ymd_and_hms(2026, 1, 1, 10, 0, 0).unwrap();
        assert_eq!(format_duration(ts, ts), "0m");
    }

    #[test]
    fn test_truncate_str_short() {
        assert_eq!(truncate_str("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_str_exact() {
        assert_eq!(truncate_str("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_str_long() {
        assert_eq!(truncate_str("hello world", 8), "hello w…");
    }

    #[test]
    fn test_visible_columns_full() {
        let cols = visible_columns(200);
        assert_eq!(cols.len(), 7);
    }

    #[test]
    fn test_visible_columns_hide_branch_first() {
        // Total of all 7 columns: 16+12+40+18+18+10+10 = 124. If < 124, Branch first hidden.
        let cols = visible_columns(123);
        assert!(!cols.contains(&COL_BRANCH));
        assert!(cols.contains(&COL_DURATION)); // Duration still visible
        assert!(cols.contains(&COL_ISSUE)); // Issue still visible
    }

    #[test]
    fn test_visible_columns_hide_duration_second() {
        // After hiding Branch (124-18=106), if < 106, Duration hidden
        let cols = visible_columns(105);
        assert!(!cols.contains(&COL_BRANCH));
        assert!(!cols.contains(&COL_DURATION));
        assert!(cols.contains(&COL_ISSUE)); // Issue still visible
    }

    #[test]
    fn test_visible_columns_minimum() {
        let cols = visible_columns(50);
        assert_eq!(cols.len(), 4);
        assert!(cols.contains(&COL_PROJECT));
        assert!(cols.contains(&COL_SUMMARY));
        assert!(cols.contains(&COL_DATE));
        assert!(cols.contains(&COL_COST));
        assert!(!cols.contains(&COL_ISSUE)); // Hidden at minimum
    }

    #[test]
    fn test_sessions_visible_rows() {
        assert_eq!(sessions_visible_rows(24), 18);
        assert_eq!(sessions_visible_rows(6), 0);
    }

    #[test]
    fn test_max_scroll_offset() {
        assert_eq!(SessionsView::max_scroll_offset(20, 15), 5);
        assert_eq!(SessionsView::max_scroll_offset(10, 15), 0);
        assert_eq!(SessionsView::max_scroll_offset(0, 15), 0);
    }

    #[test]
    fn test_format_duration_multi_hour() {
        use chrono::{TimeZone, Utc};
        let created = Utc.with_ymd_and_hms(2026, 1, 1, 8, 0, 0).unwrap();
        let modified = Utc.with_ymd_and_hms(2026, 1, 1, 14, 15, 0).unwrap();
        assert_eq!(format_duration(modified, created), "6h 15m");
    }
}
