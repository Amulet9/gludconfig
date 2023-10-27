# Maintainer: Arthex <aarthex@proton.me>

pkgname=gludconfig-git
pkgver=r1.0.0
pkgrel=1
pkgdesc="GludConfig is a attempt to rewrite GSettings in rust with additional features."
arch=('x86_64')
url="https://github.com/Amulet9/gludconfig"
license=('MIT')
depends=('rust' 'cargo')
makedepends=('git')

source=("git+https://github.com/Amulet9/gludconfig.git")

pkgver() {
  cd "$srcdir/gludconfig"
  git describe --long --tags | sed 's/\([^-]*-g\)/r\1/;s/-/./g'
}

build() {
  cd "$srcdir/gludconfig"
  cargo build --release
}

package() {
  cd "$srcdir/gludconfig"
  install -Dm755 target/release/cli "$pkgdir/usr/bin/gludconfig"
  install -Dm755 target/release/generate_code "$pkgdir/usr/bin/gludconfig_gen"
  install -Dm755 target/release/dbus "$pkgdir/usr/bin/gludconfig_daemon"
}

# Optional: add a cleanup function if you want to remove build artifacts
# after the package is installed

clean() {
  cd "$srcdir/gludconfig"
  cargo clean
}

# Optional: add package validation functions

package() {
  # Check if the required binaries are built
  if [ ! -f "$srcdir/gludconfig/target/release/cli" ]; then
    error "Missing 'cli' binary"
    return 1
  fi

  if [ ! -f "$srcdir/gludconfig/target/release/generate_code" ]; then
    error "Missing 'generate_code' binary"
    return 1
  fi

  if [ ! -f "$srcdir/gludconfig/target/release/dbus" ]; then
    error "Missing 'dbus' binary"
    return 1
  fi
}
