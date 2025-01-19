{
  description = "Flake rofi-todo";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.flake-parts.flakeModules.easyOverlay ];

      systems = [ "x86_64-linux" "aarch64-linux" ];

      perSystem = { config, self', inputs', pkgs, system, ... }: rec {
        packages = rec {
          default = rofi-todo;
          rofi-todo = pkgs.callPackage ./package.nix { inherit inputs; };
        };

        overlayAttrs = { inherit (packages) rofi-todo; };

        devShells.default = pkgs.mkShell {
          name = "rofi-todo";
          inputsFrom = [ config.packages.default ];

          packages = with pkgs; [
            cargo
            clippy
            rust-analyzer
            rustc
            rustfmt

            (pkgs.writeShellScriptBin "rofi-todo" ''
            source ~/.config/todo/config
            rofi -modi todo -show todo "$@"
            '')
          ];

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          RUST_LOG = "info";
          RUST_BACKTRACE = "full";
          ROFI_PLUGIN_PATH = "./target/debug";
        };
      };
    };
}
