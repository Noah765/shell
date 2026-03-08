{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.treefmt = {
    url = "github:numtide/treefmt-nix";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    nixpkgs,
    treefmt,
    ...
  }: let
    eachSystem = f: nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed (x: f nixpkgs.legacyPackages.${x});

    formatter = pkgs:
      (treefmt.lib.evalModule pkgs {
        programs.alejandra.enable = true;
        programs.rustfmt.enable = true;
      }).config.build.wrapper;
  in {
    packages = eachSystem (pkgs: rec {
      default = shell;

      shell = pkgs.rustPlatform.buildRustPackage {
        name = "shell";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
      };
    });

    devShells = eachSystem (pkgs: {
      default = pkgs.mkShell {
        packages = with pkgs; [cargo libxkbcommon pkg-config (formatter pkgs)];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [pkgs.wayland];
      };
    });

    formatter = eachSystem formatter;
  };
}
