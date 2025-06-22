Output assets/vhs/sync.gif

Require komac

Set Shell pwsh

Set Theme "Catppuccin Mocha"
Set FontSize 20
Set Width 1280
Set Height 300
Set Framerate 60
Set Margin 15
Set MarginFill "#657BF2"
Set Padding 10
Set BorderRadius 10
Set WindowBar Colorful

Sleep 2s

Type "komac sync"
Env VHS "true"
Sleep 1s
Enter

Wait+Screen /Successfully merged/
Sleep 5s
