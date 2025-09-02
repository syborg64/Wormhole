# Maintainer: Axel Denis <axel.denis@epitech.eu>
# Maintainer: Julian Scott <julian.scott@epitech.eu>
# Maintainer: Ludovic De Chavagnac <ludovic.de-chavagnac@epitech.eu>
# Maintainer: Arthur Aillet <arthur.aillet@epitech.eu>

pkgname=wormhole
pkgver=0.1.0
pkgrel=1
pkgdesc='Simple decentralized file storage'
url='https://github.com/Agartha-Software/Wormhole'
license=('AGPL-3.0-only')
makedepends=(cargo git fuse3)
depends=(fuse3)
arch=('any')
source=('git+https://github.com/Agartha-Software/Wormhole.git#commit=61f0ce6541a139df33050a0c609f4886f5f98901')
b2sums=("SKIP")

prepare() {
    export RUSTUP_TOOLCHAIN=stable
    cd Wormhole
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cd Wormhole
    cargo build --frozen --release --all-features
}

check() {
    export RUSTUP_TOOLCHAIN=stable
    cd Wormhole
    cargo test --frozen --all-features
}

package() {
    cd Wormhole
    find target/release \
        -maxdepth 1 \
        -executable \
        -type f \
        -exec install -Dm0755 -t "$pkgdir/usr/bin/" {} +
}