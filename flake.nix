{
  description = "Flake rofi-todo";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin" ];

      perSystem = { config, self', inputs', pkgs, system, ... }: {
        packages = rec {
          default = rofi-todo;
          rofi-todo = pkgs.callPackage ./package.nix { inherit inputs; };
        };

        devShells.default = pkgs.mkShell {
          name = "rofi-todo";
          inputsFrom = [ config.packages.default ];

          buildInputs = with pkgs; [
            cargo
            clippy
            rust-analyzer
            rustc
            rustfmt
          ];

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          RUST_LOG = "info";
          RUST_BACKTRACE = "full";
          ROFI_PLUGIN_PATH = "./target/debug";
        };
      };
    };
}
