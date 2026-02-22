{ inputs, ... }:
{
  flake.modules.homeManager.study =
    {
      config,
      lib,
      pkgs,
      ...
    }:

    let
      cfg = config.programs.study;
      tomlFormat = pkgs.formats.toml { };
    in
    {
      options.programs.study = {
        enable = lib.mkEnableOption "study, a CLI tool for managing university course exercises";

        package = lib.mkPackageOption inputs.self.packages.${pkgs.stdenv.hostPlatform.system} "study" { };

        settings = lib.mkOption {
          type = tomlFormat.type;
          default = { };
          description = "Configuration written to {file}`~/.config/study/config.toml`.";
          example = lib.literalExpression ''
            {
              courses_dir = "~/courses";
              default_template_dir = "~/.config/study/templates";
            }
          '';
        };
      };

      config = lib.mkIf cfg.enable {
        home.packages = [ cfg.package ];

        xdg.configFile."study/config.toml" = lib.mkIf (cfg.settings != { }) {
          source = tomlFormat.generate "study-config" cfg.settings;
        };
      };
    };
}
