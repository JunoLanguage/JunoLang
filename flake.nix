{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    {
      self,
      flake-utils,
      naersk,
      nixpkgs,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk { };

        buildInputs = with pkgs; [

        ];

        nativeBuildInputs = with pkgs; [

        ];
      in
      rec {
        defaultPackage = packages.app;
        packages = {
          app = naersk'.buildPackage {
            src = ./.;
            nativeBuildInputs = nativeBuildInputs;
            buildInputs = buildInputs;
          };
          contianer = pkgs.dockerTools.buildImage {
            name = "app";
            config = {
              entrypoint = [ "${packages.app}/bin/app" ];
            };
          };
        };

        devShell = pkgs.mkShell {
          LLVM_SYS_221_PREFIX = "${pkgs.llvmPackages_22.llvm.dev}";
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          LIBCLANG_PATH = "${pkgs.llvmPackages_22.libclang.lib}/lib";
          nativeBuildInputs =
            with pkgs;
            [
              nixpkgs-fmt
              cmake
              rustc
              cargo
              clippy
              llvmPackages_22.llvm.dev
              llvmPackages_22.clang
              llvmPackages_22.bintools
              rustPlatform.rustLibSrc
              libffi
              libxml2
            ]
            ++ buildInputs
            ++ nativeBuildInputs;
        };
      }
    );
}
