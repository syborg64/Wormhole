{
  description = "build for wormhole-service with fuse";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
  };

  outputs = {nixpkgs, ...}: let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };
  in {
    devShells.${system}.build = pkgs.mkShell {
      packages = [ pkgs.fuse3 pkgs.pkg-config pkgs.cargo pkgs.gcc ];
      shellHook = ''
        cargo build --bin wormhole-service
        exit
      '';
    };
  };
}
# use with :
# nix develop .#build