{
  description = "Keypad";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustTarget = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        unstableRustTarget = pkgs.rust-bin.selectLatestNightlyWith (
          toolchain:
          toolchain.default.override {
            extensions = [
              "rust-src"
              "miri"
              "rustfmt"
            ];
          }
        );
        craneLib = (crane.mkLib pkgs).overrideToolchain rustTarget;

        rustfmt' = pkgs.writeShellScriptBin "rustfmt" ''
          exec "${unstableRustTarget}/bin/rustfmt" "$@"
        '';

        defs = pkgs.callPackage ./nix { inherit craneLib; };
      in {
        checks = { } // defs.packages // defs.checks;
        packages = { } // defs.packages;

        apps.default = flake-utils.mkApp {
          name = "keypad";
          drv = defs.packages.keypad;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [ ];

          nativeBuildInputs = [
            rustfmt'
            rustTarget

            pkgs.gitlint
            pkgs.statix
          ];
        };
      }
    );
}
