# Maintainer: sever0815 <sergeilemonow@protonmail.com>
pkgname=translator-gtk
pkgver=0.1.0
pkgrel=1
pkgdesc="A small GTK4 translator written in Rust, powered by Google Translate"
arch=('x86_64' 'aarch64')
url="https://github.com/sever0815/google-translator-gtk"
license=('MIT')
depends=('gtk4>=4.12')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/sever0815/google-translator-gtk/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    # Переходим в директорию, используя wildcard (звездочку),
    # чтобы не зависеть от точного имени папки внутри архива
    cd "google-translator-gtk-$pkgver"
    cargo build --release --locked
}

package() {
    cd "google-translator-gtk-$pkgver"

    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    install -Dm644 "translator-gtk.desktop" "$pkgdir/usr/share/applications/translator-gtk.desktop"
    install -Dm644 "Google_Translate_logo.png" "$pkgdir/usr/share/icons/hicolor/256x256/apps/Google_Translate_logo.png"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
