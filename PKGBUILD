# Maintainer: Axel Denis <axel.denis@epitech.eu>
# Maintainer: Julian Scott <julian.scott@epitech.eu>
# Maintainer: Ludovic De Chavagnac <ludovic.de-chavagnac@epitech.eu>
# Maintainer: Arthur Aillet <arthur.aillet@epitech.eu>

pkgname=wormhole
pkgver=0.1.0
pkgrel=1
pkgdesc='Simple decentralized file storage'
url='https://github.com/Agartha-Software/Wormhole'
license=('GNU AFFERO GENERAL PUBLIC LICENSE')
makedepends=(cargo git)
depends=()
arch=('any')
source=('git+https://github.com/Agartha-Software/Wormhole.git#commit=db022e9aceb9c105c8de7af495a03bec9800d74e')
b2sums=("SKIP")

prepare() {
    export RUSTUP_TOOLCHAIN=stable
    cd Wormhole
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --all-features
}

check() {
    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen --all-features
}

package() {
    find target/release \
        -maxdepth 1 \
        -executable \
        -type f \
        -exec install -Dm0755 -t "$pkgdir/usr/bin/" {} +
}