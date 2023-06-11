<h1 align="center">argo-helm-updater 📦 ⬆️</h1>

<p align="center">
  Helps you to identify outdated helm charts in your argocd instance.
  <br><br>
  <a href="https://github.com/dj95/argo-helm-updater/actions/workflows/rust.yml">
    <img alt="latest version" src="https://github.com/dj95/argo-helm-updater/actions/workflows/rust.yml/badge.svg" />
  </a>
  <a href="https://github.com/dj95/argo-helm-updater/releases">
    <img alt="latest version" src="https://img.shields.io/github/v/tag/dj95/argo-helm-updater.svg?sort=semver" />
  </a>
  <br><br>
  This tool helps you to identify and update your helm charts, that are deployed with argocd.
  It retrieves all 'Application' CRDs from the given context and namespace. Since these
  applications contain all information about the helm deployment, if one is used, this tool
  queries the given repository for the latest chart version and displays a difference, if
  a newer version is deployed.
</p>


![Demo GIF of argo-helm-updater in action](./example/demo.gif)


### 📦 Requirements

- rust

*or*

- nix
- direnv


### 🚀 Getting started

Clone the repository and make sure the dependencies are installed. You either need rust or nix installed.
With nix use either `nix-shell` or `direnv allow` up to your preferences.
After dependencies are available run `cargo install --path .` to build and install the tool.

Then you should be able to call the tool with `argo-helm-updater`. It will search for the `Application` CRD of argo
in the current configured context and namespace. Use the `--context` and `--namespace` flags to search in other
clusters and namespaces.

If you'd like to update the helm version in the cluster, run `argo-helm-updater` with the  `--update` flage.
It will prompt on each new version with a confirmation whether you'd like to update the `Application` or not.


### ❄️ Installation with nix

Add the following code to your overlays. Then argo-helm-updater can be installed from `pkgs`.

```nix
final: prev: {
  argo-helm-updater = prev.pkgs.rustPlatform.buildRustPackage rec {
    version = "0.1.0";
    pname = "argo-helm-updater";

    src = prev.fetchFromGitHub {
      owner = "dj95";
      repo = pname;
      rev = "59911696aa710e5bb2c7f77789010d10d9f5fd88";
      sha256 = "sha256-D1o9gjVoM+qlht2n9pwXCDALR1pzbCwdMP+pL0NFrGs=";
    };

    cargoSha256 = "sha256-7CYMh1PcqACfpZy4dASV3MYHoxYtTxZcXt/TFaU7aWY=";

    nativeBuildInputs = [
      prev.pkgs.pkg-config
      prev.pkgs.libiconv
      prev.pkgs.openssl
    ];

    buildInputs = [] ++ prev.pkgs.lib.optionals prev.pkgs.stdenv.isDarwin [
      prev.pkgs.darwin.apple_sdk.frameworks.Cocoa
      prev.pkgs.darwin.apple_sdk.frameworks.CoreGraphics
      prev.pkgs.darwin.apple_sdk.frameworks.Foundation
      prev.pkgs.darwin.apple_sdk.frameworks.IOKit
      prev.pkgs.darwin.apple_sdk.frameworks.Kernel
      prev.pkgs.darwin.apple_sdk.frameworks.OpenGL
      prev.pkgs.darwin.apple_sdk.frameworks.Security
    ];

    PKG_CONFIG_PATH = "${prev.pkgs.openssl.dev}/lib/pkgconfig";
  };
}
```

If you just want to try it with nix, run the following command.

```bash
nix run 'github:dj95/argo-helm-updater'
```


## 🤝 Contributing

If you are missing features or find some annoying bugs please feel free to submit an issue or a bugfix within a pull request :)


## 📝 License

© 2023 Daniel Jankowski


This project is licensed under the MIT license.


Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:


The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.


THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
