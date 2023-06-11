{
  description = "Helps you to identify outdated helm charts in your argocd instance.";

  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.follows = "rust-overlay/flake-utils";
    nixpkgs.follows = "rust-overlay/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        code = pkgs.callPackage ./. { inherit nixpkgs system rust-overlay; };
      in rec {
        packages = {
          argo-helm-updater = code.argo-helm-updater;
          all = pkgs.symlinkJoin {
            name = "all";
            paths = with code; [ argo-helm-updater ];
          };
        default = packages.all;
        };
      }
    );
}
