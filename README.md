# Translator-gtk

A small GTK4 translator written in Rust, powered by Google Translate.

> **Note:** This project is a fork of [wioletowa/translator-gtk](https://github.com/wioletowa/translator-gtk).
> All original credit goes to wioletowa.

![Translator](assets/screenshot.png)

## Features

- Google Translate — free endpoint, no API key
- Automatic language detection
- Favorite languages (starred, pinned to the top of the picker)
- Language picker with search
- Swap languages (button or **Ctrl+R**)
- Image translation (OCR) via `tesseract` — from a file or the clipboard
- Config saved to `~/.config/translator-gtk/config.json`

## Usage

```sh
translator-gtk
```

A compact floating quick-translate window:

- translates instantly while you type
- **Enter** — copy the translation and close · **Shift+Enter** — newline
- **Ctrl+R** — swap languages · **Escape** — close
- **Ctrl+V** with an image in the clipboard — OCR and translate it
- **toggle**: running `translator-gtk` again while it is open closes it — bind
  the same key to `translator-gtk` for show/hide

## Arch Linux installation

### Build from source with makepkg

```sh
# Install build dependencies
sudo pacman -S --needed base-devel clang cargo

# Clone and build
git clone https://github.com/sever0815/google-translator-gtk.git
cd google-translator-gtk
makepkg -si
```

This installs `translator-gtk` to `/usr/bin`, the desktop entry to
`/usr/share/applications/`, and the icon to `/usr/share/icons/hicolor/`.

### Install without building (pre-built package)

Download the latest `.pkg.tar.zst` from the
[releases page](https://github.com/sever0815/google-translator-gtk/releases) and
install it with:

```sh
sudo pacman -U translator-gtk-*.pkg.tar.zst
```

### Optional dependencies

| Package | Purpose |
|---------|---------|
| `tesseract` | OCR engine for image translation |
| `tesseract-data-eng` | English traineddata |
| `tesseract-data-rus` | Russian traineddata |
| `wl-clipboard` | `wl-copy` for clipboard on Wayland |

```sh
sudo pacman -S tesseract tesseract-data-eng tesseract-data-rus wl-clipboard
```

## Build manually

```sh
cargo build --release
```

Requires GTK 4.12+. For image translation install tesseract with language data:

```sh
# Gentoo (LINGUAS picks the traineddata set)
LINGUAS="en ru" emerge app-text/tesseract app-text/tessdata_fast
# Arch
pacman -S tesseract tesseract-data-eng tesseract-data-rus
```

OCR uses the selected source language when its traineddata is installed,
otherwise it falls back to every installed language at once.

`wl-copy` (`wl-clipboard`) is recommended so the Enter-copy action
survives the window closing on Wayland.

### niri

The resizable window tiles by default. To keep it floating at its base
size add a window rule:

```kdl
window-rule {
    match app-id="com.whyoolw.Translator" title="Translator"
    open-floating true
}
```

## Adding a translation service

Implement the `Provider` trait in `src/providers.rs` and wire it up in
`app::build_app` (`src/app.rs`).
