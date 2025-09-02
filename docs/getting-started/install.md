# Installing Wormhole

### Arch
You can use the AUR.
Example using yay:
```
yay -S wormhole
```
Should install both `wormhole` and `wormholed` binaries

### Nix / NixOS
This repo provides a flake for you that can install Wormhole

Add Wormhole in your flake inputs
```nix
# flake.nix
inputs = {
  ...
  wormhole.url = "github:Agartha-Software/Wormhole"; # add this in your inputs
  ...
};
```

Add the package in your configuration
```nix
# configuration.nix
environment.systemPackages = with pkgs; [
  ...
  inputs.wormhole.packages.${pkgs.system}.default # wormhole package
  ...
];
```

You can then rebuild using `nixos-rebuild switch" and should have access to both `wormhole` and `wormholed` binaries