{
  description = "Leptos + Axum + Sqlx + Tailwindcss development flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixpkgs_daisyui.url = "github:NixOS/nixpkgs/dc763d353cdf5c5cd7bf2c7af4b750960e66cce7";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    nixpkgs_daisyui,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      pkgs_daisyui = import nixpkgs_daisyui {
        inherit system;
      };
      my-tailwindcss = pkgs.nodePackages.tailwindcss.overrideAttrs (oa: {
        plugins = [pkgs_daisyui.daisyui];
      });
    in
      with pkgs; {
        devShells.default = mkShell {
          shellHook = ''
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig";
          '';
          nativeBuildInputs = [
            pkg-config
          ];
          buildInputs = [
            git
            openssl
            (rust-bin.stable.latest.default.override {
              extensions = ["rust-src" "rust-std" "rust-analyzer" "rustfmt" "clippy"];
              targets = ["x86_64-unknown-linux-gnu" "wasm32-unknown-unknown"];
            })
            trunk
            cargo-leptos
            leptosfmt
            sqlite
            sqlx-cli
            nil
            alejandra
            statix
            deadnix
            taplo
            sass
            my-tailwindcss
            tailwindcss-language-server
          ];
        };
      });
}
