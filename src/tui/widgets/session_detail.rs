//! Session detail view - displays per-request token breakdowns for a single session

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use super::overview::format_number;
use crate::tui::theme::Theme;
use crate::types::{SessionDetailEntry, SessionInfo};

/// Maximum content width (consistent with other views)
const MAX_CONTENT_WIDTH: u16 = 170;

/// Column indices for the per-request table
const COL_TIME: usize = 0;
const COL_MODEL: usize = 1;
const COL_INPUT: usize = 2;
const COL_OUTPUT: usize = 3;
const COL_CACHE: usize = 4;
const COL_COST: usize = 5;

/// Column definitions: (label, width)
const COLUMNS: [(&str, u16); 6] = [
    ("Time", 12),   // 0: COL_TIME
    ("Model", 22),  // 1: COL_MODEL
    ("Input", 14),  // 2: COL_INPUT
    ("Output", 14), // 3: COL_OUTPUT
    ("Cache", 14),  // 4: COL_CACHE
    ("Cost", 12),   // 5: COL_COST
];

/// Determine which columns are visible. Hide Cache first, then Output.
fn visible_columns(width: u16) -> Vec<usize> {
    const HIDE_ORDER: [usize; 2] = [COL_CACHE, COL_OUTPUT];

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

fn table_width_for(visible: &[usize]) -> u16 {
    visible.iter().map(|&i| COLUMNS[i].1).sum()
}

/// Session detail view showing header + per-request table
pub struct SessionDetailView<'a> {
    session: &'a SessionInfo,
    entries: &'a [SessionDetailEntry],
    scroll_offset: usize,
    theme: Theme,
}

impl<'a> SessionDetailView<'a> {
    pub fn new(
        session: &'a SessionInfo,
        entries: &'a [SessionDetailEntry],
        scroll_offset: usize,
        theme: Theme,
    ) -> Self {
        Self {
            session,
            entries,
            scroll_offset,
            theme,
        }
    }

    #[allow(dead_code)] // Used in tests
    pub fn max_scroll_offset(count: usize, visible_rows: usize) -> usize {
        count.saturating_sub(visible_rows)
    }
}

impl Widget for SessionDetailView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content_width = area.width.min(MAX_CONTENT_WIDTH);
        let x_offset = (area.width.saturating_sub(content_width)) / 2;
        let centered_area = Rect {
            x: area.x + x_offset,
            y: area.y,
            width: content_width,
            height: area.height,
        };

        let sidecar_lines = self.sidecar_line_count();

        let mut constraints = vec![
            Constraint::Length(1), // 0: Top padding
            Constraint::Length(1), // 1: Summary (bold)
            Constraint::Length(1), // 2: First prompt (muted)
            Constraint::Length(1), // 3: Metadata line (project, branch, duration, msgs, cost)
        ];

        let sidecar_start = constraints.len();
        if sidecar_lines > 0 {
            // Separator + sidecar lines
            constraints.push(Constraint::Length(1)); // sidecar separator
            constraints.push(Constraint::Length(sidecar_lines as u16)); // sidecar content
        }

        let sep1_idx = constraints.len();
        constraints.push(Constraint::Length(1)); // Separator
        let header_idx = constraints.len();
        constraints.push(Constraint::Length(1)); // Table header
        let rows_idx = constraints.len();
        constraints.push(Constraint::Fill(1)); // Request rows
        let sep2_idx = constraints.len();
        constraints.push(Constraint::Length(1)); // Separator
        let keys_idx = constraints.len();
        constraints.push(Constraint::Length(1)); // Keybindings

        let chunks = Layout::vertical(constraints).split(centered_area);

        self.render_summary(chunks[1], buf);
        self.render_first_prompt(chunks[2], buf);
        self.render_metadata(chunks[3], buf);

        if sidecar_lines > 0 {
            render_separator(chunks[sidecar_start], buf, self.theme);
            self.render_sidecar_metadata(chunks[sidecar_start + 1], buf);
        }

        render_separator(chunks[sep1_idx], buf, self.theme);

        let visible = visible_columns(centered_area.width);
        self.render_table_header(chunks[header_idx], buf, &visible);
        self.render_request_rows(chunks[rows_idx], buf, &visible);

        render_separator(chunks[sep2_idx], buf, self.theme);
        self.render_keybindings(chunks[keys_idx], buf);
    }
}

impl SessionDetailView<'_> {
    fn render_summary(&self, area: Rect, buf: &mut Buffer) {
        let summary = if self.session.summary.is_empty() {
            "(no summary)"
        } else {
            &self.session.summary
        };
        Paragraph::new(Line::from(vec![Span::styled(
            summary,
            Style::default()
                .fg(self.theme.accent())
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(Alignment::Center)
        .render(area, buf);
    }

    fn render_first_prompt(&self, area: Rect, buf: &mut Buffer) {
        let prompt = if self.session.first_prompt.is_empty() {
            "(no prompt)"
        } else {
            &self.session.first_prompt
        };
        let max_len = area.width.saturating_sub(4) as usize;
        let truncated = truncate_str(prompt, max_len);

        Paragraph::new(Line::from(vec![Span::styled(
            truncated,
            Style::default().fg(self.theme.muted()),
        )]))
        .alignment(Alignment::Center)
        .render(area, buf);
    }

    fn render_metadata(&self, area: Rect, buf: &mut Buffer) {
        let duration = format_duration(self.session.modified, self.session.created);

        let mut spans = vec![Span::styled(
            &self.session.project,
            Style::default().fg(self.theme.text()),
        )];

        if !self.session.git_branch.is_empty() {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                format!("⎇ {}", self.session.git_branch),
                Style::default().fg(self.theme.date()),
            ));
        }

        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            duration,
            Style::default().fg(self.theme.text()),
        ));
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("{} msgs", self.session.message_count),
            Style::default().fg(self.theme.text()),
        ));
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("${:.2}", self.session.total_cost_usd),
            Style::default().fg(self.theme.cost()),
        ));
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("{} requests", self.entries.len()),
            Style::default().fg(self.theme.muted()),
        ));

        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Center)
            .render(area, buf);
    }

    fn render_table_header(&self, area: Rect, buf: &mut Buffer, visible: &[usize]) {
        let tw = table_width_for(visible);
        let offset = area.width.saturating_sub(tw) / 2;
        let header_style = Style::default()
            .fg(self.theme.text())
            .add_modifier(Modifier::BOLD);

        let mut spans = Vec::new();
        for &col in visible {
            let (label, width) = COLUMNS[col];
            let formatted = if col == COL_TIME || col == COL_MODEL {
                format!("{:<width$}", label, width = width as usize)
            } else {
                format!("{:>width$}", label, width = width as usize)
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

    fn render_request_rows(&self, area: Rect, buf: &mut Buffer, visible: &[usize]) {
        let tw = table_width_for(visible);
        let offset = area.width.saturating_sub(tw) / 2;
        let start = self.scroll_offset;
        let end = (start + area.height as usize).min(self.entries.len());

        for (i, entry) in self.entries[start..end].iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.y + area.height {
                break;
            }

            self.render_request_row(
                Rect {
                    x: area.x + offset,
                    y,
                    width: tw.min(area.width),
                    height: 1,
                },
                buf,
                entry,
                visible,
            );
        }
    }

    fn render_request_row(
        &self,
        area: Rect,
        buf: &mut Buffer,
        entry: &SessionDetailEntry,
        visible: &[usize],
    ) {
        use chrono::Local;

        let mut spans = Vec::new();

        for &col in visible {
            let (text, style) = match col {
                COL_TIME => {
                    let local = entry.timestamp.with_timezone(&Local);
                    (
                        format!("{:<12}", local.format("%H:%M:%S")),
                        Style::default().fg(self.theme.date()),
                    )
                }
                COL_MODEL => {
                    let model = truncate_str(&entry.model, 22);
                    (
                        format!("{:<22}", model),
                        Style::default().fg(self.theme.accent()),
                    )
                }
                COL_INPUT => (
                    format!("{:>14}", format_number(entry.input_tokens)),
                    Style::default().fg(self.theme.text()),
                ),
                COL_OUTPUT => (
                    format!("{:>14}", format_number(entry.output_tokens)),
                    Style::default().fg(self.theme.text()),
                ),
                COL_CACHE => {
                    let cache = entry.cache_read_tokens + entry.cache_creation_tokens;
                    (
                        format!("{:>14}", format_number(cache)),
                        Style::default().fg(self.theme.text()),
                    )
                }
                COL_COST => (
                    format!("{:>12}", format!("${:.4}", entry.cost_usd)),
                    Style::default().fg(self.theme.cost()),
                ),
                _ => unreachable!(),
            };

            spans.push(Span::styled(text, style));
        }

        Paragraph::new(Line::from(spans))
            .alignment(Alignment::Left)
            .render(area, buf);
    }

    /// Count how many lines the sidecar metadata section needs
    fn sidecar_line_count(&self) -> usize {
        let meta = match &self.session.metadata {
            Some(m) => m,
            None => return 0,
        };

        let mut lines = 0;
        if meta.issue_id.is_some() {
            lines += 1;
        }
        if !meta.tags.is_empty() {
            lines += 1;
        }
        if !meta.skills_used.is_empty() {
            lines += 1;
        }
        if meta.notes.is_some() {
            lines += 1;
        }
        lines
    }

    /// Render sidecar metadata section (issue, tags, skills, notes)
    fn render_sidecar_metadata(&self, area: Rect, buf: &mut Buffer) {
        let meta = match &self.session.metadata {
            Some(m) => m,
            None => return,
        };

        let label_style = Style::default()
            .fg(self.theme.muted())
            .add_modifier(Modifier::BOLD);
        let value_style = Style::default().fg(self.theme.text());
        let accent_style = Style::default().fg(self.theme.accent());

        let mut y = area.y;

        if let Some(issue_id) = &meta.issue_id {
            let line = Line::from(vec![
                Span::styled("Issue:  ", label_style),
                Span::styled(issue_id, accent_style),
            ]);
            Paragraph::new(line).alignment(Alignment::Left).render(
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                buf,
            );
            y += 1;
        }

        if !meta.tags.is_empty() {
            let tags_str = meta.tags.join(", ");
            let line = Line::from(vec![
                Span::styled("Tags:   ", label_style),
                Span::styled(tags_str, value_style),
            ]);
            Paragraph::new(line).alignment(Alignment::Left).render(
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                buf,
            );
            y += 1;
        }

        if !meta.skills_used.is_empty() {
            let skills_str = meta.skills_used.join(" → ");
            let line = Line::from(vec![
                Span::styled("Skills: ", label_style),
                Span::styled(skills_str, value_style),
            ]);
            Paragraph::new(line).alignment(Alignment::Left).render(
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                buf,
            );
            y += 1;
        }

        if let Some(notes) = &meta.notes {
            let max_len = area.width.saturating_sub(8) as usize;
            let notes_str = truncate_str(notes, max_len);
            let line = Line::from(vec![
                Span::styled("Notes:  ", label_style),
                Span::styled(notes_str, value_style),
            ]);
            Paragraph::new(line).alignment(Alignment::Left).render(
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                buf,
            );
        }
    }

    fn render_keybindings(&self, area: Rect, buf: &mut Buffer) {
        let bindings = Paragraph::new(Line::from(vec![
            Span::styled("↑↓", Style::default().fg(self.theme.accent())),
            Span::styled(": Scroll", Style::default().fg(self.theme.muted())),
            Span::raw("  "),
            Span::styled("Esc", Style::default().fg(self.theme.accent())),
            Span::styled(": Back", Style::default().fg(self.theme.muted())),
            Span::raw("  "),
            Span::styled("?", Style::default().fg(self.theme.accent())),
            Span::styled(": Help", Style::default().fg(self.theme.muted())),
        ]))
        .alignment(Alignment::Center);

        bindings.render(area, buf);
    }
}

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

/// Compute visible rows for session detail view (base overhead without sidecar)
/// padding(1) + summary(1) + prompt(1) + metadata(1) + sep(1) + header(1) + sep(1) + keybindings(1) = 8
/// When sidecar metadata is present, additional rows are used (separator + content lines)
pub fn session_detail_visible_rows(terminal_height: u16) -> usize {
    terminal_height.saturating_sub(8) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        use chrono::{TimeZone, Utc};
        let created = Utc.with_ymd_and_hms(2026, 1, 1, 10, 0, 0).unwrap();
        let modified = Utc.with_ymd_and_hms(2026, 1, 1, 12, 30, 0).unwrap();
        assert_eq!(format_duration(modified, created), "2h 30m");
    }

    #[test]
    fn test_visible_columns_full() {
        let cols = visible_columns(200);
        assert_eq!(cols.len(), 6);
    }

    #[test]
    fn test_visible_columns_narrow() {
        let cols = visible_columns(50);
        assert_eq!(cols.len(), 4);
        assert!(cols.contains(&COL_TIME));
        assert!(cols.contains(&COL_MODEL));
        assert!(cols.contains(&COL_INPUT));
        assert!(cols.contains(&COL_COST));
    }

    #[test]
    fn test_session_detail_visible_rows() {
        assert_eq!(session_detail_visible_rows(24), 16);
        assert_eq!(session_detail_visible_rows(8), 0);
    }

    #[test]
    fn test_max_scroll_offset() {
        assert_eq!(SessionDetailView::max_scroll_offset(50, 20), 30);
        assert_eq!(SessionDetailView::max_scroll_offset(10, 20), 0);
    }
}
