{
  description = "A CLI tool for syncing git repositories via bundles over SSH";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

      perSystem =
        { pkgs, self', ... }:
        {
          packages = {
            coder = pkgs.rustPlatform.buildRustPackage {
              pname = "coder";
              version = "0.1.0";
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;
            };

            default = self'.packages.coder;
          };
        };

      flake.overlays.default = final: prev: {
        coder = self.packages.${final.system}.coder;
      };
    };
}
