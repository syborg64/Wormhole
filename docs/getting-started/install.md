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

#### To try
```nix shell github:Agartha-Software/Wormhole/#wormhole```
You will then get Wormhole on this ephemeral shell.

#### To install
Add Wormhole in your flake inputs
```nix
# flake.nix
inputs = {
  ...
  wormhole.url = "github:Agartha-Software/Wormhole"; # add this in your inputs
  ...
};
```

Then add this in your flake modules (if you wish to enable the systemd service):
```nix
modules = [
  ...
  inputs.wormhole.nixosModules.wormhole
    {
      services.wormhole.enable = true;
    }
    ...
];
```

Add the package in your configuration.
It will provide the package `wormhole` and `wormhold` (even if you didn't add the systemd service).
```nix
# configuration.nix
environment.systemPackages = with pkgs; [
  ...
  inputs.wormhole.packages.${pkgs.system}.wormhole # wormhole package
  ...
];
```

You can then rebuild using `nixos-rebuild switch" and should have access to both `wormhole` and `wormholed` binaries