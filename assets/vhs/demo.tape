Output assets/vhs/demo.gif

Require komac

Set Shell pwsh

Set Theme "Catppuccin Mocha"
Set FontSize 18
Set Width 1280
Set Height 720
Set Framerate 60
Set Margin 15
Set MarginFill "#657BF2"
Set Padding 10
Set BorderRadius 10
Set WindowBar Colorful

Sleep 2s

Type "komac update PrismLauncher.PrismLauncher --version 9.4 --urls "
Type@1ms "https://github.com/PrismLauncher/PrismLauncher/releases/download/9.4/PrismLauncher-Windows-MSVC-Setup-9.4.exe"
Type " "
Type@1ms "https://github.com/PrismLauncher/PrismLauncher/releases/download/9.4/PrismLauncher-Windows-MSVC-arm64-Setup-9.4.exe"

Env SKIP_PR_CHECK "true"
Env VHS "true"

Sleep 1s
Enter

Wait+Screen /Created with komac/

Sleep 5s

Down@500ms 2
Enter
