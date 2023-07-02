<h1><img src="https://github.com/russellbanks/Komac/raw/main/assets/logo.svg" align="left" height="39" alt="Komac logo"> Komac - Another WinGet Manifest Creator <img src="https://user-images.githubusercontent.com/74878137/217098245-7aa8957b-b34e-4cba-b822-ca7a2448c3b7.svg" align="right" height="39" alt="Komac banner"></h1>

![GitHub release (release name instead of tag name)](https://img.shields.io/github/v/release/russellbanks/komac)
![GitHub Repo stars](https://img.shields.io/github/stars/russellbanks/komac)
![Issues](https://img.shields.io/github/issues/russellbanks/Komac)
![License](https://img.shields.io/github/license/russellbanks/Komac)

Komac is an advanced CLI designed to create manifests for [WinGet Community Repository](https://github.com/microsoft/winget-pkgs).

## Installation

Komac is cross-platform and runs on Windows, Linux and macOS. A cross-platform JAR is available from
the [releases](https://github.com/russellbanks/Komac/releases).

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

### Linux & macOS

It is recommended to use the cross-platform JAR for Linux and macOS. The installers are although available for these, but don't work correctly, hence I encourage you to use the JAR by using the following command:

```bash
# Think like to prefix "java -jar" to the normal command 😉
java -jar komac.jar
```

## Commands

| Command        | Description                                                                   | Usage                                      |
| -------------- | ----------------------------------------------------------------------------- | ------------------------------------------ |
| New            | Create a package from scratch                                                 | `new`                                      |
| Update         | Update a pre-existing package in winget-pkgs                                  | `update` `up`                              |
| Remove         | Remove a version from winget-pkgs                                             | `remove` `rm`                              |
| Branch Cleanup | Deletes branches that have had a merged or closed pull request to winget-pkgs | `branch cleanup`                           |
| Token update   | Update stored GitHub OAuth token                                              | `token update`, `token up`                 |
| Token remove   | Delete stored GitHub OAuth token                                              | `token remove`, `token rm`, `token delete` |

### Update an existing package with a new version

```bash
# Without user interaction
komac update --id Package.Identifier --version 1.2.3 --urls https://www.firstUrl.com,https://www.secondUrl.com --submit

# With user interaction
komac update
```

| Parameter                            | Usage       | Notes                                                      |
| ------------------------------------ | ----------- | ---------------------------------------------------------- |
| Package Identifier                   | `--id`      |                                                            |
| Version                              | `--version` |                                                            |
| URLs                                 | `--urls`    | URLs are delimited by a comma (`,`)                        |
| Automatically submit                 | `--submit`  |                                                            |
| Token (if one is not already stored) | `--token`   | Komac will check for a `GITHUB_TOKEN` environment variable |

## Komac in Action 🎥

![Komac-demo](https://user-images.githubusercontent.com/74878137/216784291-de2d5dc8-d6f9-4bde-a059-7a1382c3940b.gif)

## Komac vs. other tools 🏆

While other manifest creation tools have made remarkable strides in the winget-pkgs community, their development pace is
notably slow and lacks the advanced detection capabilities that comes with Komac.

|                                          |   Komac    | WinGetCreate |                              YamlCreate                              |
| ---------------------------------------- | :--------: | :----------: | :------------------------------------------------------------------: |
| Parameters                               |     ✅     |      ✅      |                                  ❌                                  |
| Works without Git                        |     ✅     |      ✅      |                                  ❌                                  |
| Optimised manifest ordering [^1]         |     ✅     |      ❌      |                                  ✅                                  |
| Fully cross-platform                     |     ✅     |      ❌      |                               Limited                                |
| Full MSI value retrieval                 |     ✅     |   Partial    |                               Partial                                |
| Linux & macOS MSI support                |     ✅     |      ❌      |                                  ❌                                  |
| Full MSIX value retrieval                |     ✅     |   Partial    |      Limited - https://github.com/Trenly/winget-pkgs/issues/180      |
| Get information from GitHub              |     ✅     |      ❌      |                                  ❌                                  |
| Formatted GitHub release notes retrieval |     ✅     |      ❌      |                                  ❌                                  |
| Release date identification              |     ✅     |      ❌      |                                  ❌                                  |
| Webpage metadata scraping                |     ✅     |      ❌      |                                  ❌                                  |
| Fully standalone (w/o winget-pkgs clone) |     ✅     |      ✅      |                                  ❌                                  |
| No telemetry                             |     ✅     |      ❌      |                                  ✅                                  |
| Type-safety                              |     ✅     |      ✅      |                                  ❌                                  |
| Inno setup detection                     |     ✅     |      ✅      |                               ✅ [^2]                                |
| Nullsoft detection                       |     ✅     |      ✅      |                               ✅ [^2]                                |
| Burn installer detection                 |     ✅     |      ✅      | Opt-in feature [^2] (not enabled by default, due to slow processing) |
| Progress bar & ETA while downloading     |     ✅     |      ❌      |                                  ❌                                  |
| Language                                 | Kotlin/JVM |      C#      |                              PowerShell                              |

[^1]: If all installers have the same value, it is put at the root of the manifest to reduce redundancy.
[^2]: The logic for this was contributed by me :) Check [issues](https://github.com/Trenly/winget-pkgs/issues?q%253Dauthor%253Arussellbanks) that I've opened to request this feature for YamlCreate.

## How can I support Komac? ❤️

- 🤝 Sponsor this project through [GitHub Sponsors](https://github.com/sponsors/russellbanks)
- ⭐ Star this project! :)
- 🧑‍💻 Use Komac and [create an issue](https://github.com/russellbanks/Komac/issues/new) for feature requests or bugs.

## Powering GitHub Action: WinGet Releaser 🌟

I'm happy to say that WinGet Releaser is powered by Komac, due to its advanced detection capabilities and its ability to work cross-platform. The action is available on the [GitHub Marketplace](https://github.com/marketplace/actions/winget-releaser).

It can be used in CI/CD workflows to automatically create & publish WinGet manifests for your releases. Just make sure to add first version of your manifest manually, and the action will take care of the future versions. More information to set it up can be found on the action's repository [here](https://github.com/vedantmgoyal2009/winget-releaser).

## License

[![GNU GPLv3 Logo](https://www.gnu.org/graphics/gplv3-127x51.png)](http://www.gnu.org/licenses/gpl-3.0.en.html)

Komac is Free Software: You can use, study share and improve it at your will. Specifically you can redistribute and/or
modify it under the terms of the [GNU General Public License](http://www.gnu.org/licenses/gpl-3.0.en.html) as published
by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
