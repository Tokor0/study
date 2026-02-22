{ inputs, ... }:
{
  perSystem =
    { system, ... }:
    let
      vicinaePkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [ inputs.vicinae.overlays.default ];
      };
      study-vicinae-pkg =
        { mkVicinaeExtension }:

        mkVicinaeExtension {
          pname = "study-vicinae";
          version = "1.0.0";
          src = ../../vicinae-extension;
        };
    in
    {
      packages.study-vicinae = vicinaePkgs.callPackage study-vicinae-pkg { };
    };
}
