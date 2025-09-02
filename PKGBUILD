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
makedepends=('cargo')
depends=()
arch=('any')
source=("${pkgname}-${pkgver}.zip::https://github.com/Agartha-Software/Wormhole/archive/refs/tags/v$pkgver.zip")
b2sums=("bac907e2fd3e9a96aa2d548a026230d183c7c9e580fdfaeb2718fd5fe7bb84cb146a0df9e7fdab76b6866eeb0660fad5dcbb79d7b2ded6fea0d1c8f0270eb0d2")

prepare() {
    export RUSTUP_TOOLCHAIN=stable
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