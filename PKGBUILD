# Maintainer: sever0815 <superjoika412@tred.su>
pkgname=translator-gtk
pkgver=0.1.0
pkgrel=1
pkgdesc="A small GTK4 translator written in Rust, powered by Google Translate"
arch=('x86_64' 'aarch64')
url="https://github.com/sever0815/google-translator-gtk"
license=('MIT')
depends=('gtk4>=4.12')
makedepends=('cargo')
optdepends=(
    'tesseract: OCR image translation'
    'tesseract-data-eng: English OCR language data'
    'tesseract-data-rus: Russian OCR language data'
    'wl-clipboard: clipboard support on Wayland'
)
# Ссылка на репозиторий для сборки
source=("$pkgname-$pkgver.tar.gz::https://github.com/sever0815/google-translator-gtk/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    # Переходим в распакованную директорию
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$pkgname-$pkgver"

    # Бинарник
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"

    # Desktop файл и иконка (они уже лежат в репозитории)
    install -Dm644 "translator-gtk.desktop" "$pkgdir/usr/share/applications/translator-gtk.desktop"
    install -Dm644 "Google_Translate_logo.png" "$pkgdir/usr/share/icons/hicolor/256x256/apps/Google_Translate_logo.png"

    # Лицензия
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
