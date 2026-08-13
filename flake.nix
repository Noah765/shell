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
          "cosmic-client-toolkit-0.2.0" = "sha256-LUAmB+3+doRZOJbVURaIInaQuV/LXCKfoWHA28ihAMo=";
          "cryoglyph-0.1.0" = "sha256-10JUHl1ktbqLaReuiU3HPa4r2KvsoryyJoF3BFoge3U=";
          "dpi-0.1.2" = "sha256-8r9O5RgVa8vxkPPYvr2aQiRdZ4isg7Jdnk8O5gQIr9k=";
          "iced-0.14.0" = "sha256-qqzsmAJVKD92812vfxnvsYuvNhPS3m8AImSYzSPc/pw=";
          "smithay-clipboard-0.8.0" = "sha256-GojAFRbhJcP0Rpr+v9WOivgW9x38PZdeBWTbMhkDB3A=";
          "softbuffer-0.4.1" = "sha256-9Ret/nfieBFl4yJ9TddyWsSuS7sI4QAza/TZrxYMb+I=";
        };

        nativeBuildInputs = with pkgs; [autoPatchelfHook clang copyDesktopItems pkg-config];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [pkgs.libclang];
        desktopItems = pkgs.lib.singleton (pkgs.makeDesktopItem {
          name = "shell";
          desktopName = "shell";
        });

        buildInputs = with pkgs; [libgcc libx11 libxcb libxkbcommon pipewire];
        runtimeDependencies = [pkgs.vulkan-loader pkgs.wayland];

        meta.mainProgram = "shell";
      };
    });

    devShells = eachSystem (pkgs: let
      comet = pkgs.rustPlatform.buildRustPackage (finalAttrs: {
        pname = "comet";
        version = "c4d45e3f502d9e18e0d9d4eda2c07093c62d8309";

        src = pkgs.fetchFromGitHub {
          owner = "iced-rs";
          repo = "comet";
          rev = finalAttrs.version;
          hash = "sha256-HAZjWGTIvGYSfZVD5rBV7gA++o/a91ndHKW5OjJvTd8=";
        };
        cargoHash = "sha256-0EqHoruoyUOXVFbdjKwyEy0NghNE3G7AFxwFlYlfkPg=";

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
