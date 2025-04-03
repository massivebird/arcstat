{
  description = "Video game archive statistics generator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    # nixpkgs.url = "nixpkgs";
    naersk = {
      url = "github:nix-community/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        # for `nix build` and `nix run`:
        packages.default = naersk-lib.buildPackage ./.;

        # for `nix develop`:
        shells.default = with pkgs;
          mkShell {
            buildInputs =
              [ cargo rustc rustfmt pre-commit rustPackages.clippy ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      });
}
