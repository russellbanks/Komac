Output demo.gif

Require komac

Set Theme "Catppuccin Mocha"
Set FontSize 14
Set Width 1280
Set Height 720
Set Framerate 30
Set Padding 10
Set BorderRadius 10
Set WindowBar Colorful

Sleep 1s

Type "komac update PrismLauncher.PrismLauncher --version 9.4 --urls "
Copy "https://github.com/PrismLauncher/PrismLauncher/releases/download/9.4/PrismLauncher-Windows-MSVC-Setup-9.4.exe"
Paste
Type " "
Copy "https://github.com/PrismLauncher/PrismLauncher/releases/download/9.4/PrismLauncher-Windows-MSVC-arm64-Setup-9.4.exe"
Paste

Env SKIP_PR_CHECK "true"
Env VHS_RECORD "true"

Sleep 1s
Enter

Wait+Screen /Created with komac/

Sleep 5s

Down@500ms 2
Enter
