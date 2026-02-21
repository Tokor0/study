{
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "study";
  version = "0.1.0";

  cargoHash = "sha256-742Kz5Gz64HepbHNbQyKLVII0fAlgYG46uEuN7gyLHw=";

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.intersection (lib.fileset.fromSource (lib.sources.cleanSource ../.)) (
      lib.fileset.unions [
        ../Cargo.toml
        ../Cargo.lock
        ../src
      ]
    );
  };

  meta = {
    description = "CLI tool for managing university course exercises";
    mainProgram = "study";
  };
}
