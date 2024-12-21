<h1><img src="assets/logo.svg" align="left" height="39" alt="Komac logo">Komac - 一个 WinGet 清单创建器 <img src="assets/banner.svg" align="right" height="39" alt="Komac banner"></h1>

![GitHub release (release name instead of tag name)](https://img.shields.io/github/v/release/russellbanks/komac)
![GitHub Repo stars](https://img.shields.io/github/stars/russellbanks/komac)
![Issues](https://img.shields.io/github/issues/russellbanks/Komac)
![License](https://img.shields.io/github/license/russellbanks/Komac)

Komac 是一个高级 CLI，旨在为 [WinGet 社区仓库](https://github.com/microsoft/winget-pkgs) 创建清单。

Komac 既快速 🔥 又非常节省内存，在原作者的机器上仅使用约 3.5MB 的内存。

![Komac-demo](assets/demo.gif)

<!--
## 安装

Komac 是跨平台的，提供了适用于 Windows、Linux 和 macOS 的二进制文件。

### 所有平台

如果你已经安装了 cargo，你可以为任何平台编译 Rust

```bash
cargo install --locked komac
```

Komac 还支持 [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)：

```bash
cargo binstall komac
```

#### Nightly

每次提交都会构建的 Nightly 版本可在 [这里](https://github.com/russellbanks/Komac/releases/tag/nightly) 获取。这些版本对于调试或获取最新功能和修复非常有用。

### Windows

可以从 [releases](https://github.com/russellbanks/Komac/releases) 获取便携式 EXE 和安装程序。

#### WinGet

```bash
winget install komac
```

#### Scoop

```bash
scoop install komac
```

### Linux

可以从 [releases](https://github.com/russellbanks/Komac/releases) 获取便携式二进制文件。还提供了 Debian (`.deb`) 和 Red Hat (`.rpm`) 安装程序。

### macOS

可以从 [releases](https://github.com/russellbanks/Komac/releases) 获取 macOS 的便携式二进制文件。

#### Homebrew

```bash
brew install russellbanks/tap/komac
```-->

## GitHub 令牌

Komac 目前只能使用经典令牌。虽然 Komac 可以使用细粒度令牌完全创建清单并提交，但它无法创建到 winget-pkgs 的拉取请求。随着细粒度令牌的改进，这种情况可能会改变。参见 https://github.com/russellbanks/Komac/issues/310。

### 经典

具有 `public_repo` 范围的经典令牌。

![firefox_IYiqtsd0Nl](https://github.com/russellbanks/Komac/assets/74878137/fbe4472e-dc53-4caf-ad2b-3bef75c47b07)

## 命令

| 命令            | 描述                                                                                               | 用法                      |
|-----------------|----------------------------------------------------------------------------------------------------|---------------------------|
| New             | 从头创建一个包                                                                                     | `new`                     |
| Update          | 更新 winget-pkgs 中的预先存在的包                                                                  | `update`                  |
| Remove          | 从 winget-pkgs 中删除一个版本                                                                      | `remove`                  |
| Sync Fork       | 将你的 winget-pkgs 分叉同步到 [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs)    | `sync-fork`, `sync`       |
| Branch Cleanup  | 删除已合并或关闭的拉取请求的分支                                                                   | `cleanup`                 |
| List Versions   | 列出给定包的所有版本                                                                               | `list-versions`, `list`   |
| Analyse         | 分析文件并输出信息。对调试很有用                                                                   | `analyse`                 |
| Token update    | 更新存储的 GitHub OAuth 令牌                                                                       | `token update`            |
| Token remove    | 删除存储的 GitHub OAuth 令牌                                                                       | `token remove`            |
| Complete        | 为给定的 shell 输出自动完成脚本                                                                   | `complete`, `autocomplete`|

### 使用新版本更新现有包

```
komac update Package.Identifier --version 1.2.3 --urls https://www.firstUrl.com https://www.secondUrl.com --submit
```

| 参数                                | 用法                               | 备注                                                      |
|-------------------------------------|------------------------------------|-----------------------------------------------------------|
| 包标识符                            | `komac update Package.Identifier`  |                                                           |
| 版本                                | `--version`                        |                                                           |
| URL                                 | `--urls`                           | URL 以空格分隔                                            |
| 自动提交                            | `--submit`                         |                                                           |
| 令牌（如果尚未存储）                | `--token`                          | Komac 将检查 `GITHUB_TOKEN` 环境变量                      |

## Komac 与其他工具的比较 🏆

虽然其他清单创建工具为 winget-pkgs 中的清单奠定了坚实的基础，但它们的开发速度明显较慢，缺乏 Komac 所具备的高级检测功能。

|                                          | Komac  | WingetCreate |                           YamlCreate                           |
|------------------------------------------|:------:|:------------:|:--------------------------------------------------------------:|
| 参数                                     |   ✅    |      ✅       |                               ❌                                |
| 下载进度条和预计时间                     |   ✅    |      ❌       |                               ❌                                |
| 完全跨平台                               |   ✅    |      ❌       |                            有限支持                            |
| 无需 Git 工作                            |   ✅    |      ✅       |                               ❌                                |
| 完整的 Inno Setup 值检索                 |   ✅    |      ❌       |                               ❌                                |
| 完整的 MSI 值检索                        |   ✅    |   部分支持   |                            部分支持                            |
| Linux 和 macOS 的 MSI 支持               |   ✅    |      ❌       |                               ❌                                |
| 完整的 MSIX 值检索                       |   ✅    |   部分支持   |   部分支持 - https://github.com/Trenly/winget-pkgs/issues/180   |
| 从 GitHub 获取信息                       |   ✅    |      ✅       |                               ❌                                |
| 格式化的 GitHub 发布说明检索             |   ✅    |      ❌       |                               ❌                                |
| 发布日期识别                             |   ✅    |      ❌       |                               ❌                                |
| 无遥测                                   |   ✅    |    ⭕ [^1]    |                               ✅                                |
| 完全独立（无需 winget-pkgs 克隆）        |   ✅    |      ✅       |                               ❌                                |
| Inno setup 检测                          | ✅ [^2] |      ✅       |                             ✅ [^3]                             |
| Nullsoft 检测                            | ✅ [^2] |      ✅       |                             ✅ [^3]                             |
| Burn 安装程序检测                        | ✅ [^2] |      ✅       | 选择加入功能（默认未启用，因为处理速度较慢）                   |
| 编程语言                                 |  Rust  |      C#      |                           PowerShell                           |

[^1]: WingetCreate 默认启用遥测。使用 `wingetcreate settings` 手动禁用遥测。
[^2]: 自 Komac v2 以来，Inno、Nullsoft 和 Burn 安装程序的检测更加准确。
[^3]: 该逻辑由原作者贡献 :)

查看 [issues](https://github.com/Trenly/winget-pkgs/issues?q=is:issue+author:russellbanks) 了解原作者为 YamlCreate 请求此功能的情况。

## 我如何支持 Komac？❤️

- 🤝 通过 [GitHub Sponsors](https://github.com/sponsors/russellbanks) 赞助这个项目的原作者。
- ⭐ 给这个项目加星！ :)
- 🧑‍💻 使用 Komac 并 [创建问题(源仓库)](https://github.com/russellbanks/Komac/issues/new) 提出功能请求或报告错误。

<!--## Star 历史 ⭐

<a href="https://star-history.com/#russellbanks/Komac&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=russellbanks/Komac&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=russellbanks/Komac&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=russellbanks/Komac&type=Date" />
  </picture>
</a>-->

## 许可证

[![GNU GPLv3 Logo](https://www.gnu.org/graphics/gplv3-127x51.png)](http://www.gnu.org/licenses/gpl-3.0.en.html)

Komac 是自由软件：你可以随意使用、研究、分享和改进它。具体来说，你可以根据自由软件基金会发布的 [GNU 通用公共许可证](http://www.gnu.org/licenses/gpl-3.0.en.html) 的条款重新分发和/或修改它，许可证版本为 3，或（由你选择）任何更高版本。
