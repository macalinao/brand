{ pkgs, ... }:

{
  packages = with pkgs; [
    git
  ];

  languages.rust.enable = true;

  scripts.brand.exec = ''
    cargo run --release -p brand -- "$@"
  '';

  scripts.brand-debug.exec = ''
    cargo run -p brand -- "$@"
  '';

  git-hooks.hooks = {
    clippy = {
      enable = true;
      settings = {
        allFeatures = true;
        denyWarnings = true;
        extraArgs = "--all-targets";
      };
    };
    rustfmt.enable = true;
    nixfmt.enable = true;
  };
}
