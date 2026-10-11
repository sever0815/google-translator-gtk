# Maintainer: sever0815 <sergeilemonow@protonmail.com>
pkgname=translator-gtk
pkgver=0.1.0
pkgrel=1
pkgdesc="A small GTK4 translator written in Rust, powered by Google Translate"
arch=('x86_64' 'aarch64')
url="https://github.com/sever0815/google-translator-gtk"
license=('MIT')
depends=('gtk4>=4.12')
makedepends=('cargo' 'gcc' 'clang')
optdepends=(
    'tesseract: OCR image translation'
    'tesseract-data-eng: English OCR language data'
    'tesseract-data-rus: Russian OCR language data'
    'wl-clipboard: clipboard support on Wayland'
)
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$pkgname-$pkgver"
    export CC="clang"
    export CXX="clang++"
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="clang"
    export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
    cargo build --release --locked
}

package() {
    cd "$pkgname-$pkgver"

    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    install -Dm644 "$pkgname.desktop" "$pkgdir/usr/share/applications/$pkgname.desktop"
    install -Dm644 "Google_Translate_logo.png" \
        "$pkgdir/usr/share/icons/hicolor/256x256/apps/Google_Translate_logo.png"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
