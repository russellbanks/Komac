# Assets

## Colours

|          **#1E81FF**          |             **#A975E5**             |                        **Gradient**                        |
|:-----------------------------:|:-----------------------------------:|:----------------------------------------------------------:|
| ![Royal blue](royal_blue.svg) | ![Medium purple](medium_purple.svg) | ![Gradient from royal blue to medium purple](gradient.svg) |

## Banner font

The font used in the banner is [Quicksand](https://fonts.google.com/specimen/Quicksand).

<img src="banner.svg" alt="komac banner" height="100" />

The spacing between the letters has been reduced in [Inkscape](https://inkscape.org/) to -32px.

## [ICO](https://en.wikipedia.org/wiki/ICO_(file_format)) file

The `.ico` file is created using [ImageMagick](https://imagemagick.org/) to convert from the [logo svg](logo.svg):

```powershell
magick -background none logo.svg -define icon:auto-resize="256,48,32,24,16" logo.ico
```

|                         **16x16**                          |                         **24x24**                          |                         **32x32**                          |                         **48x48**                          |                          **256x256**                          |
|:----------------------------------------------------------:|:----------------------------------------------------------:|:----------------------------------------------------------:|:----------------------------------------------------------:|:-------------------------------------------------------------:|
| <img src="logo.svg" alt="komac logo at 16x16" width="16"/> | <img src="logo.svg" alt="komac logo at 24x24" width="24"/> | <img src="logo.svg" alt="komac logo at 32x32" width="32"/> | <img src="logo.svg" alt="komac logo at 48x48" width="48"/> | <img src="logo.svg" alt="komac logo at 256x256" width="256"/> |

The icon sizes are
the [minimum recommended icon sizes](https://learn.microsoft.com/windows/apps/design/style/iconography/app-icon-construction#icon-scaling)
an app should have.
