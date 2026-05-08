{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
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

      shell = pkgs.rustPlatform.buildRustPackage {
        pname = "shell";
        version = "0.1.0";

        src = ./.;

        cargoLock.lockFile = ./Cargo.lock;
        cargoLock.outputHashes = {
          "accesskit-0.22.0" = "sha256-pP9CyiV1zIONQ7vbl5MkMtilemSPrHaZ0c/SyR+lb0k=";
          "clipboard_macos-0.1.0" = "sha256-WO3JFbE+6ESRAfkxrnEFeZyGuhUHLOKOVHcGQyHwoK0=";
          "cosmic-client-toolkit-0.2.0" = "sha256-ymn+BUTTzyHquPn4hvuoA3y1owFj8LVrmsPu2cdkFQ8=";
          "cosmic-text-0.19.0" = "sha256-sQJN7WtWAesesUEprd+oDQ19XtaWwWvbY5qrNJXLks0=";
          "cryoglyph-0.1.0" = "sha256-sSfgXlWgrM4wdczdquqzc/uuUmHL/GuK+Xvn0XNO+UQ=";
          "dpi-0.1.2" = "sha256-pvGeHgfGetFutV2Pr39Jse+REFOmCkI1djzHqMQcWmE=";
          "iced-0.14.0" = "sha256-LMZkt3iZec0w2OHtLEjHk3jMV57Et/BbRhIFC4RA+O0=";
          "smithay-clipboard-0.8.0" = "sha256-GojAFRbhJcP0Rpr+v9WOivgW9x38PZdeBWTbMhkDB3A=";
          "softbuffer-0.4.1" = "sha256-9Ret/nfieBFl4yJ9TddyWsSuS7sI4QAza/TZrxYMb+I=";
        };

        nativeBuildInputs = with pkgs; [autoPatchelfHook clang pkg-config];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [pkgs.libclang];

        buildInputs = with pkgs; [libgcc libx11 libxcb libxkbcommon pipewire];
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

        buildInputs = [pkgs.libgcc];
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
