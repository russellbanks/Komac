### Komac's user-facing text. This is the source every other language is translated from.
###
### Syntax: https://projectfluent.org/fluent/guide/
### Message IDs are kebab-case and grouped by area. Arguments such as { $identifier } are filled
### in by Komac and must be kept, but can be moved anywhere in the sentence. Argument names are
### snake_case, as they're also Rust identifiers.

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
prompt-installer-url =
    { NUMBER($position, type: "ordinal") ->
        [one] { $position }st
        [two] { $position }nd
        [few] { $position }rd
       *[other] { $position }th
    } Installer URL
    .invalid = Please enter a valid URL
    .help = Press ESC if you do not have any more URLs
prompt-portable-exe = Is { $file } a portable exe?
prompt-nested-files = Select the nested files
prompt-nested-portable-command-alias = Portable command alias for { $path }:
prompt-removal-reason = Give a reason for removing { $identifier } version { $version }
prompt-remove-version = Would you like to make a pull request to remove { $identifier } { $version }?
# $status is the HTTP status, such as "404 Not Found"
prompt-remove-dead-version = { $identifier } { $version } returned { $status } in all its InstallerUrls. Remove?
prompt-remove-anyway = Remove anyway?
prompt-remove-token = Would you like to remove the currently stored token?
prompt-proceed = Would you like to proceed?
prompt-token = Enter a GitHub token
prompt-new-token = Please enter the new token to set
prompt-select-branches = Please select branches to delete
prompt-select-packages = Please select which packages to submit
prompt-submit-option = What would you like to do with { $identifier } { $version }?
    .submit = Submit
    .edit = Edit
    .exit = Exit

## Terminal output

# Highlighted in green and passed as { $success } to messages that report success
success = Successfully
latest-version = Latest version of { $identifier }: { $version }
manifests-written = { $success } written all manifest files to { $path }
# $state is "open", "merged" or "closed"
pull-request-exists = There is already { $state ->
        [open] an open
        [merged] a merged
       *[closed] a closed
    } pull request for { $identifier } { $version } that was created on { $date } at { $time }
pull-request-closed-proceeding = Proceeding as the existing pull request was closed without being merged
pull-request-creating = Creating a pull request for { $identifier } { $version }
pull-request-creating-removal = Creating a pull request to remove { $identifier } { $version }
pull-request-created = { $success } created { $pull_request }
pull-request-rate-limit = Last pull request was created { $elapsed } ago. Waiting for { $remaining }
downloading = Downloading { $file }
version-count = There { $count ->
        [one] is { $count } version
       *[other] are { $count } versions
    }
no-packages-to-submit = No valid packages to submit were found in { $path }
update-available = A new release of { $name } is available:

## cleanup. $state is "merged", "closed" or "any".

cleanup-retrieving = Retrieving branches that have a { $state ->
        [merged] merged
        [closed] closed
       *[any] merged or closed
    } pull request associated with them
cleanup-no-branches = There are no { $state ->
        [merged] merged
        [closed] closed
       *[any] merged or closed
    } pull requests with branches that can be deleted
cleanup-none-deleted = No branches have been deleted
cleanup-deleting = Deleting { $count } selected { $count ->
        [one] branch
       *[other] branches
    }
cleanup-deleted = { $success } deleted { $count } selected { $count ->
        [one] branch
       *[other] branches
    }

## remove-version and remove-dead-versions

remove-version-warning = Packages should only be removed when necessary
remove-dead-checking = Checking { $identifier } { $version }
# $status is the HTTP status, such as "404 Not Found", and $state is "open", "merged" or "closed"
remove-dead-pull-request-exists = { $identifier } { $version } returned { $status } in all its InstallerUrls but there is already { $state ->
        [open] an open
        [merged] a merged
       *[closed] a closed
    } pull request for this version that was created on { $date } at { $time }.

## sync

commits = { $count ->
        [one] { $count } commit
       *[other] { $count } commits
    }
# $commits is a number of commits from the `commits` message
merge-summary = { $commits } from { $upstream } into { $fork }
sync-up-to-date = { $fork } is already { $status } with { $upstream }
    .status = up-to-date
sync-fork-ahead = { $fork } is not behind { $upstream }, so there is nothing to merge. It has { $commits } that { $upstream_name } does not
# $summary is from the `merge-summary` message, and $kind is "merge" or "fast-forward"
sync-merged = { $success } merged { $summary } ({ $kind ->
        [merge] merge
       *[fast-forward] fast-forward
    })
sync-merging = Merging { $summary }

## token

token-none-stored = No token is currently stored in the platform's secure storage
token-deleted = { $success } deleted the stored token from the platform's secure storage
token-not-deleted = No token was deleted
token-stored = { $success } stored token in platform's secure storage
