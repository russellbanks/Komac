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

## Manifest editor. Arguments such as { $quit } are key names, which are shown in bold.

editor-help = Press { $quit } to quit, { $save } to save, { $search } to search, { $switch } to switch buffer
editor-search-help = Press { $enter } to jump to first match and close, { $close } to close, { $next } to search next, { $previous } to search previous
editor-search = Search
editor-search-error = Search: { $error }
editor-pattern-not-found = Pattern not found
editor-switched-buffer = Switched to buffer #{ $number }
editor-saved = Saved!
# Shown after the file name when it has unsaved changes
editor-modified = [modified]

## Command-line help. The message is shown by -h, and `.long` by --help when present.

command-komac = A manifest creator for winget-pkgs
command-analyze = Analyzes a file and outputs information about it
command-cleanup = Finds branches from the fork of the upstream repository that have had a merged or closed pull request to the upstream repository from them, prompting for which ones to delete
command-complete = Outputs an autocompletion script for the given shell. Example usage:
    .long =
        Outputs an autocompletion script for the given shell. Example usage:

        Bash: echo "source <(komac complete bash)" >> ~/.bashrc
        Elvish: echo "eval (komac complete elvish | slurp)" >> ~/.elvish/rc.elv
        Fish: echo "source (komac complete fish | psub)" >> ~/.config/fish/config.fish
        PowerShell: echo "komac complete powershell | Out-String | Invoke-Expression" >> $PROFILE
        Zsh: echo "source <(komac complete zsh)" >> ~/.zshrc
command-list = Lists all versions for a given package
command-new = Create a new package from scratch
command-remove = Remove a version from the upstream repository (microsoft/winget-pkgs by default)
    .long =
        Remove a version from the upstream repository (microsoft/winget-pkgs by default)

        To remove a package, all versions of that package must be removed
command-show = Output the manifests for a given package and version
command-sync = Merges changes from the upstream repository (microsoft/winget-pkgs by default) into the fork repository
command-token-remove = Remove the stored token
command-token-update = Update the stored token
command-update = Add a version to a pre-existing package

arg-analyze-no-hash = Skip hashing the file
arg-analyze-hash = Hash the file and include it in the `InstallerSha256` field
arg-cleanup-all = Automatically delete all relevant branches
arg-cleanup-only-closed = Only delete closed branches
arg-cleanup-only-merged = Only delete merged branches
arg-complete-shell = Specifies the shell for which to generate the completion script
    .long =
        Specifies the shell for which to generate the completion script.

        If not provided, the shell will be inferred based on the current environment.
arg-concurrent-downloads = Number of installers to download at the same time
arg-created-with = Name of external tool that invoked Komac
arg-created-with-url = URL to external tool that invoked Komac
arg-dry-run = Run without submitting
arg-exit = Automatically skip the prompt and exit (with saving the manifests if --output is defined)
arg-fast = Use the per-minute rate limit, potentially hitting the hourly rate limit in 7.5 minutes
arg-identifier = The package's unique identifier
arg-list-count = Output the number of versions the package has
arg-list-json = Output the versions as JSON
arg-list-pretty-json = Output the versions as prettified JSON
arg-list-yaml = Output the versions as YAML
arg-new-dry-run = Run without prompting or submitting
arg-new-resolves = List of issues that adding this package or version would resolve
arg-open-pr = Open pull request link automatically
arg-output = Directory to output the manifests to
arg-package-identifier = The package's unique identifier
arg-package-version = The package's version
arg-remove-dead-versions-after = Check versions greater than a given version
arg-remove-dead-versions-auto = Automatically create pull requests to remove dead versions without prompting
arg-remove-dead-versions-before = Check versions lesser than a given version
arg-remove-dead-versions-concurrent = Number of versions to check concurrently
arg-remove-no-warning = Don't show the package removal warning
arg-remove-resolves = List of issues that removing this version would resolve
arg-repository = The GitHub repository that manifests are read from and pull requests are opened against
    .long =
        The GitHub repository that manifests are read from and pull requests are opened against

        Use this to work with a third-party or private repository instead of microsoft/winget-pkgs. Your fork must have the same name as this repository, except where a command lets you name the fork, such as `sync --fork`.
arg-show-default-locale-manifest = Switch to display the default locale manifest
arg-show-installer-manifest = Switch to display the installer manifest
arg-show-locale-manifests = Switch to display all locale manifests
arg-show-version-manifest = Switch to display the version manifest
arg-skip-pr-check = Skip checking for existing pull requests
arg-submit = Automatically submit a pull request
arg-submit-resolves = List of issues that submitting this version would resolve
arg-submit-skip-prompt = Skip the confirmation prompt to submit the package
arg-submit-submit-all = Submit all packages present in the folder
arg-sync-branch = The branch to sync. Without this, the fork's default branch is synced with upstream's default branch. With it, the branch is synced with upstream's branch of the same name
arg-sync-fork = The fork to sync, if it is not `<you>/<upstream name>`, e.g. a renamed fork or one owned by an organization
arg-sync-fork-owner = The owner of the fork, if it is not the authenticated user. Ignored when --fork is given
arg-token = GitHub personal access token with the `public_repo` scope
arg-token-remove-skip-prompt = Skip the confirmation prompt to delete the token
arg-token-update-token = The new token to store
arg-update-release-notes-url = URL to package's release notes
arg-update-replace = Package version to replace
arg-update-resolves = List of issues that updating this package would resolve
arg-urls = Direct download URLs of the package's installers, separated by a space
arg-version = The package's version
