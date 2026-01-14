# VHS documentation: https://github.com/charmbracelet/vhs

Output demo.gif

Set FontSize 14
Set Width 1200
Set Height 800
Set TypingSpeed 50ms
Set Theme "GruvboxDarkHard"

Type "cargo run --quiet -- --niri-config tests/niri-default-config.kdl | fzf"
Enter
Sleep 1s

Type "focus-wo"
Sleep 1s

Down
Sleep 300ms
Down
Sleep 300ms
Down
Sleep 300ms
Up
Sleep 300ms
Up
Sleep 1s

Ctrl+C
Sleep 500ms
