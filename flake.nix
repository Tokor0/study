{
  description = "study — CLI tool for managing university course exercises";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    vicinae.url = "github:vicinaehq/vicinae";
  };

  outputs =
    inputs@{ self, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = inputs.nixpkgs.lib.systems.flakeExposed;
      perSystem =
        {
          pkgs,
          system,
          ...
        }:
        let
          vicinaPkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.vicinae.overlays.default ];
          };
        in
        {
          packages = {
            study = pkgs.callPackage ./nix/package.nix { };
            default = pkgs.callPackage ./nix/package.nix { };
            study-courses-extension = vicinaPkgs.callPackage ./nix/vicinae-extension.nix { };
          };

          devShells.default = pkgs.mkShell {
            inputsFrom = [ (pkgs.callPackage ./nix/package.nix { }) ];
          };
        };
      flake = {
        homeManagerModules.study = import ./nix/hm-module.nix self;
        homeManagerModules.default = self.homeManagerModules.study;
      };
    };
}
