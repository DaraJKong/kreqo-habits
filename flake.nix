{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
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
            trunk
            sqlite
            sass
            openssl
            (rust-bin.nightly.latest.default.override {
              extensions = ["rust-src"];
              targets = ["wasm32-unknown-unknown"];
            })
          ];
        };
      });
}
