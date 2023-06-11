{ pkgs, ... }:
{
  argo-helm-updater = pkgs.rustPlatform.buildRustPackage {
    pname = "argo-helm-updater";
    version = "0.1.0";
    src = ./.;

    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.libiconv
      pkgs.openssl
    ];

    buildInputs = [] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
      pkgs.darwin.apple_sdk.frameworks.Cocoa
      pkgs.darwin.apple_sdk.frameworks.CoreGraphics
      pkgs.darwin.apple_sdk.frameworks.Foundation
      pkgs.darwin.apple_sdk.frameworks.IOKit
      pkgs.darwin.apple_sdk.frameworks.Kernel
      pkgs.darwin.apple_sdk.frameworks.OpenGL
      pkgs.darwin.apple_sdk.frameworks.Security
    ];

    PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
  };
}
