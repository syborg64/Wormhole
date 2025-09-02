{
  inputs = { nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05"; };

  outputs = { nixpkgs, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forEachSystem = nixpkgs.lib.genAttrs systems;
    in {
      devShells = forEachSystem (system:
        let pkgs = nixpkgs.legacyPackages.${system};
        in {
          default = pkgs.mkShell {
            packages = [
              pkgs.cargo
              pkgs.rustc
              pkgs.rustfmt
              pkgs.pkg-config
              pkgs.fuse3
              pkgs.docker
              pkgs.docker-compose
            ];
            RUST_SRC_PATH =
              "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        });

      packages = forEachSystem (system:
        let pkgs = nixpkgs.legacyPackages.${system};
        in {
          # https://github.com/NixOS/nixpkgs/blob/4fc53b59aecbc25c0e173163d60155f8fca14bfd/doc/languages-frameworks/rust.section.md
          default = pkgs.stdenv.mkDerivation {
            pname = "wormhole";
            version = "0.1.0";

            src = fetchGit {
              url = "https://github.com/Agartha-Software/Wormhole";
              rev = "7c1ad13d3c590c2717e0a75e6ec366067491f830";
            };

            buildInputs = [ pkgs.rustc pkgs.cargo pkgs.fuse3 pkgs.pkg-config ];

            # preBuild = ''
            #   export RUST_SRC_PATH=${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}
            #   export RUSTUP_TOOLCHAIN=stable
            #   export CARGO_TARGET_DIR=target
            #   cargo fetch --locked --target ${system}
            # '';

            buildPhase = ''
              runHook preBuild
              export CARGO_HOME=$PWD/.cargo
              export RUST_SRC_PATH=${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}
              export RUSTUP_TOOLCHAIN=stable
              export CARGO_TARGET_DIR=target
              cargo build --frozen --release --all-features
            '';

            installPhase = ''
              mkdir -p $out/bin
              cp target/release/wormhole $out/bin/
              cp target/release/wormholed $out/bin/
            '';

            meta = {
              description = "My handwritten package";
              license = pkgs.lib.licenses.mit;
            };
          };
        });
    };
}
