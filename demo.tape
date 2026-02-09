Output assets/demo.gif
Set Width 1200
Set Height 600
Set FontSize 14
Set Theme "Homebrew"

# 실행
Type "toktrack"
Enter
Sleep 1500ms

# Overview: 소스 탐색 (j로 이동)
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 1000ms

# 첫 번째 소스로 돌아가서 SourceDetail 진입
Type "k"
Sleep 200ms
Type "k"
Sleep 200ms
Type "k"
Sleep 200ms
Enter
Sleep 1500ms

# SourceDetail: Daily 스크롤
Type "k"
Sleep 200ms
Type "k"
Sleep 200ms
Type "k"
Sleep 200ms
Type "k"
Sleep 200ms
Type "k"
Sleep 1000ms

# Model breakdown popup
Enter
Sleep 2000ms
Escape
Sleep 500ms

# Weekly 뷰
Type "w"
Sleep 1500ms

# Monthly 뷰
Type "m"
Sleep 1500ms

# Overview로 돌아가기
Escape
Sleep 1000ms

# Tab 2: Stats
Type "2"
Sleep 1500ms

# Tab 3: Models
Type "3"
Sleep 1500ms

# 종료 (Ctrl+C → y)
Ctrl+C
Sleep 500ms
Type "y"
