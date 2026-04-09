{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
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
        src = ./.;
        strictDeps = true;

        nativeBuildInputs = with pkgs; [autoPatchelfHook clang pkg-config];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [pkgs.libclang];
        autoPatchelfIgnoreMissingDeps = ["libgcc_s.so.1"];

        buildInputs = with pkgs; [libx11 libxcb libxkbcommon pipewire];
        runtimeDependencies = [pkgs.vulkan-loader pkgs.wayland];

        meta.mainProgram = "shell";
      };
    });

    devShells = eachSystem (pkgs: let
      comet = pkgs.rustPlatform.buildRustPackage (finalAttrs: {
        pname = "comet";
        version = "fbef808eed51562f0ea601d8fc7c715bea9cfd0b";

        src = pkgs.fetchFromGitHub {
          owner = "iced-rs";
          repo = "comet";
          rev = finalAttrs.version;
          hash = "sha256-aefw4FK40Nu7+hOJ0geOpYg/XXFEFmdCD3x2xrVEHVk=";
        };
        cargoHash = "sha256-c3at2XyG2c+mJD43YMlfolT1WZaDcBzfxXoS0CX8lag=";

        nativeBuildInputs = [pkgs.autoPatchelfHook];
        autoPatchelfIgnoreMissingDeps = ["libgcc_s.so.1"];

        runtimeDependencies = with pkgs; [libxkbcommon vulkan-loader wayland];
      });
    in {
      default = pkgs.mkShell {
        packages = with pkgs; [
          cargo
          clang
          clippy
          comet
          libx11
          libxcb
          libxkbcommon
          pipewire
          pkg-config
          rust-analyzer
          rustc
          (formatter pkgs)
        ];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [libclang vulkan-loader wayland]);
      };
    });

    formatter = eachSystem formatter;
  };
}
