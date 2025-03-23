{ pkgs, craneLib }:

let
  inherit (pkgs) lib;
  src = craneLib.cleanCargoSource ../.;

  commonArgs = {
    inherit src;
    strictDeps = true;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  individualCrateArgs = commonArgs // {
    inherit cargoArtifacts;
    inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;

    doCheck = false;
  };

  fileSetForCrate =
    crate:
    lib.fileset.toSource {
      root = ../.;
      fileset = lib.fileset.unions [
        ../Cargo.lock
        ../Cargo.toml
        (craneLib.fileset.commonCargoSources crate)
      ];
    };
in
{
  packages = {
    keypad = craneLib.buildPackage (
      individualCrateArgs
      // {
        pname = "keypad";
        cargoExtraARgs = "-p cloudmqtt";
        src = fileSetForCrate ./..;
      }
    );
  };

  checks = {
    workspace-clippy = craneLib.cargoClippy (
      commonArgs
      // {
        inherit cargoArtifacts;
        cargoClippyExtraArgs = "--all-targets -- --deny warnings";
      }
    );

    workspace-doc = craneLib.cargoDoc (
      commonArgs
      // {
        inherit cargoArtifacts;
      }
    );

    workspace-fmt = craneLib.cargoFmt {
      inherit src;
    };
  };
}

