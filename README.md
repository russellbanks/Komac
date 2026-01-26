<h2 align="center">
  <img height="100" alt="komac" src="assets/branding/banner.svg" />
  <br>
  Komac: Smart Manifest Automation for WinGet
</h2>

<div align="center">

[![Latest version](https://img.shields.io/github/v/release/russellbanks/komac?label=Latest&color=A975E5)](https://github.com/russellbanks/komac/releases/latest)
[![Build](https://img.shields.io/github/actions/workflow/status/russellbanks/komac/build.yml?label=Build&color=647BF2)](https://github.com/russellbanks/komac/actions)
[![GitHub Issues](https://img.shields.io/github/issues/russellbanks/komac?label=Issues&color=417EF9)](https://github.com/russellbanks/Komac/issues)
[![License](https://img.shields.io/crates/l/komac?label=License&color=1E81FF)](https://github.com/russellbanks/Komac/blob/main/LICENSE.md)

</div>

<div align="center">

[![Crates.io Version](https://img.shields.io/crates/v/komac)](https://crates.io/crates/komac)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/komac?label=Downloads%20from%20crates.io)](https://crates.io/crates/komac)
[![MSRV](https://img.shields.io/crates/msrv/komac?label=MSRV)](https://github.com/russellbanks/Komac/blob/main/Cargo.toml)

</div>

<p align="center">
  A <strong>fast, intelligent CLI</strong> to create and submit <a href="https://github.com/microsoft/winget-pkgs">WinGet manifests</a>.
  <br>
  Think: automation, deep installer analysis, and zero YAML wrangling.
</p>


<div align="center">
  <img src="assets/vhs/demo.gif" alt="Demo gif" />
</div>

## ⚡ Quick start

```bash
# 1. Install komac
winget install komac
  
# 2. Set your GitHub token
komac token add
  
# 3. Update a package
komac update Package.Identifier --version 1.2.3 --urls https://example.com/installer.exe
```

## 🎯 Features

- 🔄 Advanced installer analysis
    - [Inno Setup](https://jrsoftware.org/isinfo.php)
    - [Nullsoft Scriptable Install System](https://nsis.sourceforge.io)
    - [MSI](https://learn.microsoft.com/windows/win32/msi/windows-installer-portal)
    - [Burn](https://docs.firegiant.com/wix/tools/burn/)
- 🌍 Cross-platform support (Windows, Linux, macOS, Android)

## Installation

Komac is cross-platform and binaries are built for Windows, Linux, macOS, and Android.

### All platforms

If you have cargo installed, you can compile Rust for any platform:

```bash
cargo install --locked komac
```

Komac also supports [cargo-binstall](https://github.com/cargo-bins/cargo-binstall):

```bash
cargo binstall komac
```

#### Nightly

Nightly releases that are built every week are
available [here](https://github.com/russellbanks/Komac/releases/tag/nightly).
These are useful for debugging or if you want the latest features and fixes.

### Windows

Both portable EXEs and installers are available from the [releases](https://github.com/russellbanks/Komac/releases).

#### WinGet

```bash
winget install komac
```

#### Scoop

```bash
scoop install komac
```

### Linux

Portable binaries are available from the [releases](https://github.com/russellbanks/Komac/releases).
Debian (`.deb`)and Red Hat (`.rpm`) installers are also available.

### macOS

Portable binaries for macOS are available from the [releases](https://github.com/russellbanks/Komac/releases).

#### Homebrew

```bash
brew install komac
```

### Android [(Termux App)](https://termux.dev/en/)

#### 1) Install [Termux-User-Repository (TUR)](https://github.com/termux-user-repository/tur) in Termux

```bash
pkg install tur-repo
```

#### 2) Install Komac in Termux

```bash
pkg install komac
```

## GitHub Token Setup

komac needs a classic GitHub token to submit PRs to `winget-pkgs`.

This should be a classic token with the `public_repo` scope.

![Screenshot of classic token being created with public_repo scope selected](https://github.com/russellbanks/komac/assets/74878137/fbe4472e-dc53-4caf-ad2b-3bef75c47b07)

Adding to komac:

```bash
komac token add
```

The token can be also be obtained using the [GitHub CLI](https://cli.github.com/):

```bash
komac token add --token=$(gh auth token)
```

<details>

<summary>Why not fine-grained tokens?</summary>

Whilst Komac can fully create manifests and commit with a fine-grained token, it fails to create a pull request to
winget-pkgs. This may change as fine-grained tokens improve.
See https://github.com/russellbanks/Komac/issues/310.

</details>

## Commands

<details>

<summary>Full Command List</summary>

| Command        | Description                                                                                         | Usage                      |  
|----------------|-----------------------------------------------------------------------------------------------------|----------------------------|  
| New            | Create a package from scratch                                                                       | `new`                      |  
| Update         | Update a pre-existing package in winget-pkgs                                                        | `update`                   |  
| Remove         | Remove a version from winget-pkgs                                                                   | `remove`                   |  
| Sync Fork      | Syncs your fork of winget-pkgs to [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) | `sync-fork`, `sync`        |  
| Branch Cleanup | Deletes branches that have had a merged or closed pull request to winget-pkgs                       | `cleanup`                  |  
| List Versions  | Lists all the versions for a given package                                                          | `list-versions`, `list`    |  
| Analyse        | Analyses a file and outputs information about it. Useful for debugging                              | `analyse`                  |  
| Token update   | Update stored GitHub OAuth token                                                                    | `token update`             |  
| Token remove   | Delete stored GitHub OAuth token                                                                    | `token remove`             |  
| Complete       | Outputs an autocompletion script for the given shell                                                | `complete`, `autocomplete` |  

</details>

### komac new

Creates a new package from scratch.

```bash
komac new
```

<div align="center">
  <img src="assets/vhs/new_package.gif" alt="New package gif" />
</div>

### komac update

Add a version to a pre-existing package:

```
komac update Package.Identifier --version 1.2.3 --urls https://www.example.com/installer.exe https://www.example.com/installer.msi --submit
```

| Parameter                            | Usage                             | Notes                                                      |
|--------------------------------------|-----------------------------------|------------------------------------------------------------|
| Package Identifier                   | `komac update Package.Identifier` |                                                            |
| Version                              | `--version`                       |                                                            |
| URLs                                 | `--urls`                          | URLs are delimited by a space                              |
| Automatically submit                 | `--submit`                        |                                                            |
| Token (if one is not already stored) | `--token`                         | Komac will check for a `GITHUB_TOKEN` environment variable |

### komac sync

Updates your fork of winget-pkgs to be up-to-date
with [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs).

```bash
komac sync
```

<div align="center">
  <img src="assets/vhs/sync.gif" alt="Sync gif" />
</div>

## Feature Comparison 🔍

While other manifest creation tools have made a solid foundation for the manifests in winget-pkgs, their development
pace
is slower and lacks the deeper installer analysis that komac is capable of.

|                                          | Komac   | WingetCreate  |                           YamlCreate                            |
|------------------------------------------|:-------:|:-------------:|:---------------------------------------------------------------:|
| Parameters                               |   ✅    |      ✅       |                               ❌                                |
| Download progress bar & ETA              |   ✅    |      ❌       |                               ❌                                |
| Fully cross-platform                     |   ✅    |      ❌       |                            Limited                              |
| Works without Git                        |   ✅    |      ✅       |                               ❌                                |
| Full Inno Setup value retrieval          |   ✅    |      ❌       |                               ❌                                |
| Full MSI value retrieval                 |   ✅    |   Partial     |                            Partial                              |
| Linux, macOS & Android MSI support       |   ✅    |      ❌       |                               ❌                                |
| Full MSIX value retrieval                |   ✅    |   Partial     |   Partial - https://github.com/Trenly/winget-pkgs/issues/180    |
| Get information from GitHub              |   ✅    |      ✅       |                               ❌                                |
| Formatted GitHub release notes retrieval |   ✅    |      ❌       |                               ❌                                |
| Release date identification              |   ✅    |      ❌       |                               ❌                                |
| No telemetry                             |   ✅    |    ⭕ [^1]    |                               ✅                                |
| Fully standalone (w/o winget-pkgs clone) |   ✅    |      ✅       |                               ❌                                |
| Inno setup detection                     | ✅ [^2] |      ✅       |                             ✅ [^3]                             |
| Nullsoft detection                       | ✅ [^2] |      ✅       |                             ✅ [^3]                             |
| Burn installer detection                 | ✅ [^2] |      ✅       | Opt-in feature (not enabled by default due to slow processing)  |
| Programming Language                     |  Rust   |      C#       |                           PowerShell                            |

[^1]: Telemetry is enabled by default in WingetCreate. Use `wingetcreate settings` to manually disable telemetry.
[^2]: There is much more accurate detection for Inno, Nullsoft, and Burn installers since Komac v2.
[^3]: The logic for this was contributed by me :)
Check [issues](https://github.com/Trenly/winget-pkgs/issues?q=is:issue+author:russellbanks) that I've opened to request
this feature for YamlCreate.

## GitHub Actions 🌟

[WinGet Releaser](https://github.com/vedantmgoyal9/winget-releaser) is a GitHub Action that invokes Komac, passing in
your release's URLs. This completely automates
publishing to WinGet.

### Example 📝

```yaml
name: Publish to WinGet
on:
  release:
    types: [ released ]
jobs:
  publish:
    runs-on: windows-latest
    steps:
      - uses: vedantmgoyal9/winget-releaser@main
        with:
          identifier: Package.Identifier
          token: ${{ secrets.WINGET_TOKEN }}
          # installers-regex: '\.exe$' # Only .exe files
```

### Alternative actions 🔄

- Run Komac manually: [michidk/run-komac](https://github.com/michidk/run-komac)
- Automate releases for external repositories: [michidk/winget-updater](https://github.com/michidk/winget-updater)

## How can I support Komac? ❤️

- ⭐ Star this project! :)
- 🤝 Sponsor this project through [GitHub Sponsors](https://github.com/sponsors/russellbanks)
- 🧑‍💻 Use Komac and [create an issue](https://github.com/russellbanks/Komac/issues/new) for feature requests or bugs.

## Star History ⭐

<a href="https://star-history.com/#russellbanks/Komac&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=russellbanks/Komac&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=russellbanks/Komac&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=russellbanks/Komac&type=Date" />
  </picture>
</a>

## License

[![GNU GPLv3 Logo](https://www.gnu.org/graphics/gplv3-or-later.png)](http://www.gnu.org/licenses/gpl-3.0.en.html)

Komac is Free Software: You can use, study share and improve it at your will. Specifically you can redistribute and/or
modify it under the terms of the [GNU General Public License](http://www.gnu.org/licenses/gpl-3.0.en.html) as published
by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
