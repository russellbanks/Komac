### Komac's user-facing text. This is the source every other language is translated from.
###
### Syntax: https://projectfluent.org/fluent/guide/
### Message IDs are kebab-case and grouped by area. Arguments such as { $identifier } are filled
### in by Komac and must be kept, but can be moved anywhere in the sentence.

## Manifest fields, used to label prompts. `.help` is shown below a prompt, `.placeholder` inside
## an empty one, and `.plural` labels a prompt that takes a list.

field-author = Author
field-channel = Channel
field-command = Command
    .plural = Commands
    .help = List of commands or aliases to run the package
field-copyright = Copyright
    .help = Example: Copyright (c) Microsoft Corporation
field-copyright-url = Copyright URL
field-custom-switch = Custom switch
    .help = Example: /norestart, -norestart
field-description = Description
field-file-extension = File extension
    .plural = File extensions
    .help = List of file extensions the package could support
field-installation-notes = Installation notes
field-installer-return-code = Installer return code
    .plural = Installer return codes
    .help = List of additional non-zero installer success exit codes other than known default values by winget
field-install-modes = Install modes
field-language-tag = Language tag
    .help = Example: en-US
field-license = License
    .help = Example: MIT, GPL-3.0, Freeware, Proprietary
field-license-url = License URL
field-moniker = Moniker
    .help = Example: vscode
field-package-identifier = Package identifier
    .help = Package Identifiers are in the format of Publisher.package
    .placeholder = Publisher.package
field-package-name = Package name
    .help = Example: Microsoft Teams
field-package-url = Package URL
field-package-version = Package version
    .help = Example: 1.2.3
field-portable-command-alias = Portable command alias
field-protocol = Protocol
    .plural = Protocols
    .help = List of protocols the package provides a handler for. Example: http, https
field-publisher = Publisher
    .help = Example: Microsoft Corporation
field-publisher-support-url = Publisher support URL
field-publisher-url = Publisher URL
field-release-notes-url = Release notes URL
field-short-description = Short description
field-silent-switch = Silent switch
    .help = Example: /S, -verysilent, /qn, --silent, /exenoui
field-silent-with-progress-switch = Silent with progress switch
    .help = Example: /S, -silent, /qb, /exebasicui
field-tag = Tag
    .help = Example: zip, c++, photos, OBS
field-upgrade-behavior = Upgrade behavior

## Prompts

prompt-too-many-items = There is a maximum of { $max } items. There were { $count } provided
