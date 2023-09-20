{ pkgs ? import <nixpkgs> {} }:
with pkgs;
let
  pinnedPkgs = fetchFromGitHub {
    owner = "NixOS";
    repo = "nixpkgs";
    rev = "5148520bfab61f99fd25fb9ff7bfbb50dad3c9db";
    sha256 = "sha256-d2B282GmQ9o8klc22/Rbbbj6r99EnELQpOQjWMyv0rU=";
  };

  pkgs = import pinnedPkgs {};

  inherit (lib) optional optionals;
  inherit (darwin.apple_sdk.frameworks) Cocoa CoreGraphics Foundation IOKit Kernel OpenGL Security;
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo
    cargo-audit
    clippy
    libiconv
    rustc
    openssl
    pkg-config
  ] ++ optionals stdenv.isDarwin [
    Cocoa
    CoreGraphics
    Foundation
    IOKit
    Kernel
    OpenGL
    Security
    libpng
    zlib
  ];
}
