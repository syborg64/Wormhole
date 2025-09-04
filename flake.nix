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
        let
          pkgs = nixpkgs.legacyPackages.${system};
          # The following sources helps downloading the custom winfsp patch
          # If winfsp in one day sourced on the official repo, could dismiss
          # this for a single derivation, like was started on this commit:
          # d968432c3c9b38ddb35da9a20e79dc0a31cf1e74
          aggregatedSource = pkgs.stdenv.mkDerivation {
            pname = "cargo-crates-dl";
            version = "0.0.0";
            src = fetchGit {
              url = "https://github.com/Agartha-Software/Wormhole";
              rev = "65510c91fc23c72e97b318e2a0d79bc5d01e9f51";
            };
            doCheck = false;
            dontFixup = true;
            nativeBuildInputs = with pkgs; [ cargo rustc cacert wget ];
            buildPhase = ''
              runHook preBuild
              export CARGO_HOME=$PWD/.cargo
              cargo fetch --locked
              runHook postBuild
            '';
            installPhase = ''
              runHook preInstall
              mkdir $out
              cp -r . $out
              runHook postInstall
            '';
            outputHashAlgo = "sha256";
            outputHashMode = "recursive";
            outputHash = "sha256-O2l82Ezrgxh1P7PXbXoo0Pon9zwyAzFmspT6tqta0so=";
          };
        in {
          default = pkgs.stdenv.mkDerivation {
            pname = "wormhole";
            version = "0.1.0";

            src = aggregatedSource;

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
              description = "Simple decentralized file storage";
              license = pkgs.lib.licenses.agpl3Only;
            };
          };
        });
    };
}
