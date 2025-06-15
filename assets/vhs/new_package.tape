Output assets/vhs/new_package.gif

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

Type "komac new"
Env SKIP_PR_CHECK "true"
Env VHS "true"
Sleep 1s
Enter

# Package Identifier
Wait+Line /Package identifier/
Sleep 0.5s
Type "Jackett.Jackett"
Sleep 1s
Enter

# Package Version
Wait+Line /Package version/
Sleep 0.5s
Type "0.22.2020"
Sleep 1s
Enter

# 1st Installer URL
Wait+Line /1st Installer URL/
Sleep 0.5s
Type@1ms "https://github.com/Jackett/Jackett/releases/download/v0.22.2020/Jackett.Installer.Windows.exe"
Sleep 1s
Enter

# 2nd Installer URL
Wait+Line /2nd Installer URL/
Sleep 1s
Escape

# Language tag
Wait+Line /Language tag/
Sleep 0.5s
Type "en-US"
Sleep 1s
Enter

# Installer return code
Wait+Line /Installer return code/
Sleep 1s
Enter

# Upgrade Behavior
Wait+Line /Upgrade behavior/
Sleep 1s
Enter

# Command
Wait+Line /Command/
Sleep 1s
Enter

# Protocol
Wait+Line /Protocol/
Sleep 1s
Enter

# File extension
Wait+Line /File extension/
Sleep 1s
Enter

# Publisher URL
Wait+Line /Publisher URL/
Sleep 0.5s
Type@1ms "https://github.com/Jackett/Jackett"
Sleep 1s
Enter

# Publisher support URL
Wait+Line /Publisher support URL/
Sleep 0.5s
Type@1ms "https://github.com/Jackett/Jackett/issues"
Sleep 1s
Enter

# Author
Wait+Line /Author/
Sleep 0.5s
Type "Jackett Contributors"
Sleep 1s
Enter

# Package URL
Wait+Line /Package URL/
Sleep 0.5s
Type@1ms "https://github.com/Jackett/Jackett"
Sleep 1s
Enter

# License URL
Wait+Line /License URL/
Sleep 0.5s
Type@1ms "https://github.com/Jackett/Jackett/blob/HEAD/LICENSE"
Sleep 1s
Enter

# Copyright
Wait+Line /Copyright/
Sleep 0.5s
Type "Copyright (c) Jackett"
Sleep 1s
Enter

# Copyright URL
Wait+Line /Copyright URL/
Sleep 1s
Enter

# Short description
Wait+Line /Short description/
Sleep 0.5s
Type "API Support for your favorite torrent trackers"
Sleep 1s
Enter

# Description
Wait+Line /Description/
Sleep 1s
Enter

# Moniker
Wait+Line /Moniker/
Sleep 0.5s
Type "jackett"
Sleep 1s
Enter

# Release Notes URL
Wait+Line /Release notes URL/
Sleep 0.5s
Type@1ms "https://github.com/Jackett/Jackett/releases/tag/v0.22.2020"
Sleep 1s
Enter

Wait+Screen /Created with komac/

Sleep 5s

Down@500ms 2
Enter
