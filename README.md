<h1><img src="assets/logo.svg" align="left" height="39" alt="Komac logo">Komac - Another WinGet Manifest Creator <img src="assets/banner.svg" align="right" height="39" alt="Komac banner"></h1>

![GitHub release (release name instead of tag name)](https://img.shields.io/github/v/release/russellbanks/komac)
![GitHub Repo stars](https://img.shields.io/github/stars/russellbanks/komac)
![Issues](https://img.shields.io/github/issues/russellbanks/Komac)
![License](https://img.shields.io/github/license/russellbanks/Komac)

Komac is an advanced CLI designed to create manifests for
the [WinGet Community Repository](https://github.com/microsoft/winget-pkgs).

Komac is both blazingly fast 🔥 and incredibly low on memory, using just ~3.5MB of memory on my machine.

![Komac-demo](assets/demo.gif)

## Installation

Komac is cross-platform and binaries are built for Windows, Linux, macOS, and FreeBSD.

### All platforms

If you have cargo installed, you can compile Rust for any platform

```bash
cargo install komac
```

Komac also supports [cargo-binstall](https://github.com/cargo-bins/cargo-binstall):

```bash
cargo binstall komac
```

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

Portable binaries are available from the [releases](https://github.com/russellbanks/Komac/releases). Debian (`.deb`) and
Red Hat (`.rpm`) installers are also available.

### macOS

Portable binaries for macOS are available from the [releases](https://github.com/russellbanks/Komac/releases).

#### Homebrew

```bash
brew install russellbanks/tap/komac
```

## GitHub Token

Komac is currently only able to use a classic token. Whilst Komac can fully create manifests and commit with a
fine-grained token, it fails to create a pull request to winget-pkgs. This may change as fine-grained tokens improve.
See https://github.com/russellbanks/Komac/issues/310.

### Classic

A classic token with the `public_repo` scope.

![firefox_IYiqtsd0Nl](https://github.com/russellbanks/Komac/assets/74878137/fbe4472e-dc53-4caf-ad2b-3bef75c47b07)

## Commands

| Command        | Description                                                                   | Usage          |
|----------------|-------------------------------------------------------------------------------|----------------|
| New            | Create a package from scratch                                                 | `new`          |
| Update         | Update a pre-existing package in winget-pkgs                                  | `update`       |
| Remove         | Remove a version from winget-pkgs                                             | `remove`       |
| Branch Cleanup | Deletes branches that have had a merged or closed pull request to winget-pkgs | `cleanup`      |
| Token update   | Update stored GitHub OAuth token                                              | `token update` |
| Token remove   | Delete stored GitHub OAuth token                                              | `token remove` |

### Update an existing package with a new version

```bash
komac update -i Package.Identifier -v 1.2.3 --urls https://www.firstUrl.com https://www.secondUrl.com --submit
```

| Parameter                            | Usage          | Notes                                                      |
|--------------------------------------|----------------|------------------------------------------------------------|
| Package Identifier                   | `--identifier` |                                                            |
| Version                              | `--version`    |                                                            |
| URLs                                 | `--urls`       | URLs are delimited by a space                              |
| Automatically submit                 | `--submit`     |                                                            |
| Token (if one is not already stored) | `--token`      | Komac will check for a `GITHUB_TOKEN` environment variable |

## Komac vs other tools 🏆

While other manifest creation tools have made remarkable strides in the winget-pkgs community, their development pace is
notably slow and lacks the advanced detection capabilities that come with Komac.

|                                          | Komac  | WingetCreate |                           YamlCreate                           |
|------------------------------------------|:------:|:------------:|:--------------------------------------------------------------:|
| Parameters                               |   ✅    |      ✅       |                               ❌                                |
| Works without Git                        |   ✅    |      ✅       |                               ❌                                |
| Optimised manifest ordering [^1]         |   ✅    |      ✅️      |                               ✅                                |
| Fully cross-platform                     |   ✅    |      ❌       |                            Limited                             |
| Full MSI value retrieval                 |   ✅    |   Partial    |                            Partial                             |
| Linux & macOS MSI support                |   ✅    |      ❌       |                               ❌                                |
| Full MSIX value retrieval                |   ✅    |   Partial    |   Partial - https://github.com/Trenly/winget-pkgs/issues/180   |
| Get information from GitHub              |   ✅    |      ❌       |                               ❌                                |
| Formatted GitHub release notes retrieval |   ✅    |      ❌       |                               ❌                                |
| Release date identification              |   ✅    |      ❌       |                               ❌                                |
| No telemetry                             |   ✅    |    ⭕ [^2]    |                               ✅                                |
| Fully standalone (w/o winget-pkgs clone) |   ✅    |      ✅       |                               ❌                                |
| Type-safety                              |   ✅    |      ✅       |                               ❌                                |
| Inno setup detection                     | ✅ [^3] |      ✅       |                             ✅ [^4]                             |
| Nullsoft detection                       | ✅ [^3] |      ✅       |                             ✅ [^4]                             |
| Burn installer detection                 | ✅ [^3] |      ✅       | Opt-in feature (not enabled by default due to slow processing) |
| Progress bar & ETA while downloading     |   ✅    |      ❌       |                               ❌                                |
| Programming Language                     |  Rust  |      C#      |                           PowerShell                           |

[^1]: If all installers have the same value, that value is put at the root of the manifest to reduce redundancy.
[^2]: Telemetry is enabled by default in WingetCreate. Use `wingetcreate settings` to manually disable telemetry.
[^3]: There is much more accurate detection for Inno, Nullsoft, and Burn installers since Komac v2.
[^4]: The logic for this was contributed by me :)
Check [issues](https://github.com/Trenly/winget-pkgs/issues?q=is:issue+author:russellbanks) that I've opened to request
this feature for YamlCreate.

## Usage with GitHub Actions: [WinGet Releaser](https://github.com/vedantmgoyal2009/winget-releaser) 🌟

WinGet Releaser is a GitHub Action that invokes Komac, passing in your release's URLs. This completely automates
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
      - uses: vedantmgoyal2009/winget-releaser@v2
        with:
          identifier: Package.Identifier
          token: ${{ secrets.WINGET_TOKEN }}
          # installers-regex: '\.exe$' # Only .exe files
```

### Alternative actions 🔄

- Run Komac manually: [michidk/run-komac](https://github.com/michidk/run-komac)
- Automate releases for external repositories: [michidk/winget-updater](https://github.com/michidk/winget-updater)

## How can I support Komac? ❤️

- 🤝 Sponsor this project through [GitHub Sponsors](https://github.com/sponsors/russellbanks)
- ⭐ Star this project! :)
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

[![GNU GPLv3 Logo](https://www.gnu.org/graphics/gplv3-127x51.png)](http://www.gnu.org/licenses/gpl-3.0.en.html)

Komac is Free Software: You can use, study share and improve it at your will. Specifically you can redistribute and/or
modify it under the terms of the [GNU General Public License](http://www.gnu.org/licenses/gpl-3.0.en.html) as published
by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
