<br/>
<p align="center">
  <h3 align="center">Komac</h3>
<p>

<p align="center">
    The Kotlin Manifest Creator for WinGet
    <br/>
    <br/>
    <a href="https://github.com/russellbanks/Komac/issues">Report Bug</a>
    .
    <a href="https://github.com/russellbanks/Komac/issues">Request Feature</a>
</p>

![Issues](https://img.shields.io/github/issues/russellbanks/Komac)
![License](https://img.shields.io/github/license/russellbanks/Komac)

## About The Project

Komac is a manifest creator for WinGet. It stands for **KO**tlin **MA**nifest **C**reator.

🎉 Komac is currently in it's final stages of development!

Komac creates WinGet 1.4 manifests, ready for when they start getting accepted to winget-pkgs!

Below is a development demo of Komac:

![Komac-demo](https://user-images.githubusercontent.com/74878137/212578049-9d929028-daa5-47fc-8beb-2d91a1a44970.gif)

## Why should I use Komac?

Komac allows you to create WinGet manifests for applications with minimal effort. Manifest creation shouldn't be something that's only for long-time winget-pkgs contributors; it's for everyone.

## What about other manifest creation tools? Why Komac over those?

Other manifest creation tools are great and have created a solid basis for everything so far in winget-pkgs, but development of those tools is arguably very slow and lacks the advanced detction that Komac has.

Komac is also written in Kotlin, meaning it can run on any operating system, not just Windows!

For example, if you enter an MSI as an installer, Komac is able to identify the InstallerType, Version, Name, Language, UpgradeCode, UpgradeBehaviour and more, meaning the user simply doesn't need to be asked for these.

This is similar for MSIX's, APPX's, MSIXBundles, APPXBundles, Zips, etc, whereby Komac will be doing all it can to detect as much as it can from those types. Other tools just don't have this same advanced detection for these file types.

As another (of many things Komac detects) example, if you enter a Url and it contains x64 (or x86, arm, i686, i386, x86_x64, etc) within it, we can't guarantee that this is what the installer actually is, but 99% of the time it will be, so we can show this to the user and even use it as a default value for the prompt.

Finally, Komac is community-oriented. I'm a sole developer who's passionate about programming and the WinGet Package Manager Community repository.

## How can I support Komac?

- ⭐ Star this project! :)
- 🧑‍💻 Use Komac and [create an issue](https://github.com/russellbanks/Komac/issues/new) if you run into any bugs or inconsistencies

## License

[![GNU GPLv3 Logo](https://www.gnu.org/graphics/gplv3-127x51.png)](http://www.gnu.org/licenses/gpl-3.0.en.html)

Komac is Free Software: You can use, study share and improve it at your will. Specifically you can redistribute and/or modify it under the terms of the [GNU General Public License](http://www.gnu.org/licenses/gpl-3.0.en.html) as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
