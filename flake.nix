{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    cargo-hot.url = "github:hecrj/cargo-hot";
    cargo-hot.flake = false;
    comet.url = "github:iced-rs/comet";
    comet.flake = false;
    treefmt.url = "github:numtide/treefmt-nix";
    treefmt.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs: let
    eachSystem = f: inputs.nixpkgs.lib.genAttrs inputs.nixpkgs.lib.systems.flakeExposed (x: f inputs.nixpkgs.legacyPackages.${x});

    formatter = pkgs:
      (inputs.treefmt.lib.evalModule pkgs {
        programs.alejandra.enable = true;
        programs.rustfmt.enable = true;
      }).config.build.wrapper;
  in {
    packages = eachSystem (pkgs: rec {
      default = shell;

      shell = (inputs.crane.mkLib pkgs).buildPackage {
        src = (inputs.crane.mkLib pkgs).cleanCargoSource ./.;
        strictDeps = true;
        buildInputs = with pkgs; [libx11 libxcb libxkbcommon];
        nativeBuildInputs = [pkgs.pkg-config];
      };
    });

    devShells = eachSystem (pkgs: let
      cargo-hot = pkgs.rustPlatform.buildRustPackage {
        pname = "cargo-hot";
        version = inputs.cargo-hot.rev;
        src = inputs.cargo-hot;
        cargoHash = "sha256-Cvn6/HgIBqkcu7/SY2AXWio1k0vaYbvmg8EpO1TaOeE=";
        buildInputs = [pkgs.openssl];
        nativeBuildInputs = [pkgs.pkg-config];
      };
      comet = pkgs.rustPlatform.buildRustPackage {
        pname = "comet";
        version = inputs.comet.rev;
        src = inputs.comet;
        cargoHash = "sha256-c3at2XyG2c+mJD43YMlfolT1WZaDcBzfxXoS0CX8lag=";
        nativeBuildInputs = [pkgs.autoPatchelfHook];
        runtimeDependencies = with pkgs; [libxkbcommon vulkan-loader wayland];
        autoPatchelfIgnoreMissingDeps = ["libgcc_s.so.1"];
      };
    in {
      default = pkgs.mkShell {
        packages = with pkgs; [cargo cargo-hot comet libx11 libxcb libxkbcommon pkg-config (formatter pkgs)];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [pkgs.vulkan-loader pkgs.wayland];
      };
    });

    formatter = eachSystem formatter;
  };
}
