pkgname=nskm
pkgver=$( awk '$1 == "version" {split($3, x, "\""); print x[2]}' Cargo.toml )
pkgrel=1
arch=(any)
license=(GPL3)
makedepends=(cargo)

package() {
    cd "$startdir"
    cargo build --release
    install -D "$startdir"/target/release/nskm "$pkgdir"/usr/bin/nskm
    install -Dm644 "$startdir"/nskm.service "$pkgdir"/usr/lib/systemd/system/nskm@.service
    install -Dm644 "$startdir"/nskm.rules "$pkgdir"/usr/lib/udev/rules.d/91-nskm.rules
}
