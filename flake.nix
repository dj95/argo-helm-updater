{
  description = "Helps you to identify outdated helm charts in your argocd instance.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        craneLib = (crane.mkLib pkgs);

        argo-helm-updater = craneLib.buildPackage {
          src = craneLib.cleanCargoSource (craneLib.path ./.);

          doNotSign = true;

          buildInputs = [
            # Add additional build inputs here
            pkgs.pkg-config
            pkgs.libiconv
            pkgs.openssl
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.darwin.apple_sdk.frameworks.Cocoa
            pkgs.darwin.apple_sdk.frameworks.CoreGraphics
            pkgs.darwin.apple_sdk.frameworks.Foundation
            pkgs.darwin.apple_sdk.frameworks.IOKit
            pkgs.darwin.apple_sdk.frameworks.Kernel
            pkgs.darwin.apple_sdk.frameworks.OpenGL
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.libpng
            pkgs.zlib
          ];
        };
      in {
        checks = {
          inherit argo-helm-updater;
        };

        packages.default = argo-helm-updater;

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [
            cargo-audit
            cargo-edit
            cargo-watch
            clippy
            libiconv
          ];
        };
      }
    );
}
