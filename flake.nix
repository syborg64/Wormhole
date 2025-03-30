{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {nixpkgs, flake-utils, ...}:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.default = pkgs.mkShell {
        packages = [
          pkgs.cargo
          pkgs.rustc
          pkgs.pkg-config
          pkgs.fuse3
        ];
        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
#        PKG_CONFIG_PATH = "${pkgs.fuse3}/lib/pkgconfig";
#        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [pkgs.fuse3];
      };
    });
}