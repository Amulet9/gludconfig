# Maintainer: Arthex <aarthex@proton.me>

pkgname=gludconfig-git
pkgver=r1.0.0
pkgrel=1
pkgdesc="GludConfig is a attempt to rewrite GSettings in rust with additional features."
arch=('x86_64')
url="https://github.com/Amulet9/gludconfig"

source=("git+https://github.com/Amulet9/gludconfig.git")

prepare() {
    export RUSTUP_TOOLCHAIN=nightly
    cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
    
    export RUSTUP_TOOLCHAIN=nightly
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --all-features
}

package() {
  cd "$srcdir/gludconfig"
  install -Dm755 "${srcdir}/target/release/cli" "$pkgdir/usr/bin/gludconfig"
  install -Dm755 target/release/generate_code "$pkgdir/usr/bin/gludconfig_gen"
  install -Dm755 target/release/dbus "$pkgdir/usr/bin/gludconfig_daemon"
}

check() {
    export RUSTUP_TOOLCHAIN=nightly
    cargo test --frozen --all-features
}

clean() {
  remove "$srcdir/gludconfig"
}