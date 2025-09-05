{
  inputs = { nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05"; };

  outputs = { nixpkgs, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forEachSystem = nixpkgs.lib.genAttrs systems;
    in rec {
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
        in { wormhole = import ./nix/package.nix { inherit pkgs; }; });

      nixosModules.wormhole = { config, pkgs, ... }:
        let cfg = config.services.wormhole;
        in {
          options.services.wormhole = {
            enable = pkgs.lib.mkEnableOption "Run the Wormhole daemon";
            package = pkgs.lib.mkOption {
              type = pkgs.lib.types.package;
              default = packages.${pkgs.system}.wormholed;
            };
          };

          config = pkgs.lib.mkIf cfg.enable {
            systemd.services.wormhole = {
              description = "Wormhole Service Daemon";
              wantedBy = [ "multi-user.target" ];
              serviceConfig.ExecStart =
                "${cfg.package}/bin/wormholed 0.0.0.0:8081";
              serviceConfig.Restart = "on-failure";
              environment.SERVICE_ADDRESS = "0.0.0.0:8081";
            };
          };
        };
    };
}
