{
  description = "Rust nightly devShell with Cargo (edition 2024 ready)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.nightly.latest.default; # 🦀 Nightly rustc + cargo
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [ rust ];
          RUSTUP_TOOLCHAIN = "nightly";  # optional explicit environment var
        };
      }
    );
}
