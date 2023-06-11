<h1 align="center">argo-helm-updater 📦⬆️</h1>

<p align="center">
  Helps you to identify outdated helm charts in you argocd instance.
  <br><br>
  <a href="https://github.com/dj95/argo-helm-updater/releases">
    <img alt="latest version" src="https://img.shields.io/github/v/tag/dj95/argo-helm-updater.svg?sort=semver" />
  </a>
  <br><br>
  This tool helps you to identify and update your helm charts, that are deployed with argocd.
  It retrieves all `Application` CRDs from the given context and namespace. Since these
  applications contain all information about the helm deployment, if one is used, this tool
  queries the given repository for the latest chart version and displays a difference, if
  a newer version is deployed.
</p>


### 📦 Requirements

- rust

*or*

- nix
- direnv


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
