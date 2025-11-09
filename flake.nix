{
  description = "O(log n) span highlighter";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      with pkgs;
      {
        packages.default = rustPlatform.buildRustPackage (_: {
          pname = "highlight-span";
          version = "0.1.0";
          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;
        });

        devShells.default = mkShell {
          packages = [
            rustc
            cargo
            rustfmt
            rust-analyzer
          ];

          env.RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
        };
      }
    );
}
