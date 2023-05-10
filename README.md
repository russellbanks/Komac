<img src="https://user-images.githubusercontent.com/74878137/217098245-7aa8957b-b34e-4cba-b822-ca7a2448c3b7.svg" alt="Komac banner" align="right" height="60" />

# Komac: The Kotlin Manifest Creator for [winget-pkgs](https://github.com/microsoft/winget-pkgs)

![Issues](https://img.shields.io/github/issues/russellbanks/Komac)
![License](https://img.shields.io/github/license/russellbanks/Komac)

Komac is an advanced CLI designed to create manifests for [winget-pkgs](https://github.com/microsoft/winget-pkgs).

## Installation

Komac is cross-platform and runs on Windows, Linux and macOS. A cross-platform JAR is available from the [releases](https://github.com/russellbanks/Komac/releases).

### Windows

Windows EXEs are available from the [releases](https://github.com/russellbanks/Komac/releases).

#### WinGet

```bash
winget install komac
```

#### Scoop
```bash
scoop install komac
```

### Linux

A deb file for Linux is available from the [releases](https://github.com/russellbanks/Komac/releases).

### macOS

A dmg file for macOS is available from the [releases](https://github.com/russellbanks/Komac/releases).

## Usage

| Command      | Description                                                                   | Usage                                      |
|--------------|-------------------------------------------------------------------------------|--------------------------------------------|
| New          | Create a package from scratch                                                 | `new`                                      |
| Update       | Update a pre-existing package in winget-pkgs                                  | `update` `up`                              |
| Remove       | Remove a version from winget-pkgs                                             | `remove` `rm`                              |
| Cleanup      | Deletes branches that have had a merged or closed pull request to winget-pkgs | `cleanup`                                  |
| Token update | Update stored GitHub OAuth token                                              | `token update`, `token up`                 |
| Token remove | Delete stored GitHub OAuth token                                              | `token remove`, `token rm`, `token delete` |

```bash
komac [OPTIONS] COMMAND [ARGS]
```

### Update an existing package with a new version

#### Without prompts

```bash
komac update --id Package.Identifier --version 1.2.3 --urls https://www.firstUrl.com,https://www.secondUrl.com --submit
```

| Parameter                            | Usage                          | Notes                                                      |
|--------------------------------------|--------------------------------|------------------------------------------------------------|
| Package Identifier                   | `--id`, `--package-identifier` |                                                            |
| Version                              | `--version`                    |                                                            |
| URLs                                 | `--urls`                       | URLs are delimited by a comma (`,`)                        |
| Automatically submit                 | `--submit`                     |                                                            |
| Token (if one is not already stored) | `--token`                      | Komac will check for a `GITHUB_TOKEN` environment variable |

#### With prompts

```bash
komac update
```

## Komac in Action 🎥

![Komac-demo](https://user-images.githubusercontent.com/74878137/216784291-de2d5dc8-d6f9-4bde-a059-7a1382c3940b.gif)

## Komac vs other tools 🏆

While other manifest creation tools have made remarkable strides in the winget-pkgs community, their development pace is notably slow and lacks the advanced detection capabilities of Komac.

|                                          |    Komac   | WinGetCreate |                         YamlCreate                         |
|------------------------------------------|:----------:|:------------:|:----------------------------------------------------------:|
| Parameters                               |      ✅     |       ✅      |                              ❌                             |
| Works without Git                        |      ✅     |       ✅      |                              ❌                             |
| Optimised manifest ordering*             |      ✅     |       ❌      |                              ✅                             |
| Fully cross-platform                     |      ✅     |       ❌      |                           Limited                          |
| Full MSI value retrieval                 |      ✅     |    Partial   |                           Partial                          |
| Linux and macOS MSI support              |      ✅     |       ❌      |                              ❌                             |
| Full MSIX value retrieval                |      ✅     |    Partial   | Limited - https://github.com/Trenly/winget-pkgs/issues/180 |
| GitHub value retrieval                   |      ✅     |       ❌      |                              ❌                             |
| Formatted GitHub release notes retrieval |      ✅     |       ❌      |                              ❌                             |
| Release date identification              |      ✅     |       ❌      |                              ❌                             |
| Webpage metadata scraping                |      ✅     |       ❌      |                              ❌                             |
| Fully standalone                         |      ✅     |       ❌      |                              ❌                             |
| No telemetry                             |      ✅     |       ❌      |                              ✅                             |
| Type-safety                              |      ✅     |       ✅      |                              ❌                             |
| Inno setup detection                     |      ✅     |       ❌      |      https://github.com/Trenly/winget-pkgs/issues/177      |
| Nullsoft detection                       |      ✅     |       ❌      |      https://github.com/Trenly/winget-pkgs/issues/177      |
| Burn installer detection                 |      ✅     |       ❌      |      https://github.com/Trenly/winget-pkgs/issues/179      |
| Progress bar                             |      ✅     |       ❌      |                              ❌                             |
| Language                                 | Kotlin/JVM |      C#      |                         PowerShell                         |

*If all installers have the same value, it is put at the root of the manifest to reduce redundancy.

## Powering Major Repositories: Komac & WinGet Releaser 🌟

Komac isn't just a tool – it's an integral part of a larger ecosystem, playing a pivotal role in projects like [WinGet Releaser](https://github.com/vedantmgoyal2009/winget-releaser). WinGet Releaser is a GitHub action that retrieves URLs from your releases and passes them directly to Komac.

What's more, Komac is trusted by some of the most prominent and well-established repositories on GitHub, including Audacity and Vim. Our tool is enabling these major players to optimise their workflows and improve their software distribution processes.

We're immensely proud of what Komac is achieving, and we invite you to join our community, contribute, and help shape the future of Komac!

## How can I support Komac? ❤️

- ⭐ Star this project! :)
- 🧑‍💻 Use Komac and [create an issue](https://github.com/russellbanks/Komac/issues/new) if you run into any bugs or inconsistencies

## License

[![GNU GPLv3 Logo](https://www.gnu.org/graphics/gplv3-127x51.png)](http://www.gnu.org/licenses/gpl-3.0.en.html)

Komac is Free Software: You can use, study share and improve it at your will. Specifically you can redistribute and/or modify it under the terms of the [GNU General Public License](http://www.gnu.org/licenses/gpl-3.0.en.html) as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
