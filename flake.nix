{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    nix-vscode-extensions.url = "github:nix-community/nix-vscode-extensions";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    nix-vscode-extensions,
  }:
    utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
      in {
        devShell = with pkgs;
          mkShell {
            buildInputs = [
              gcc
              cargo
              rustc
              rustfmt
              rustPackages.clippy
              rust-analyzer

              (vscode-with-extensions.override {
                vscodeExtensions = with nix-vscode-extensions.extensions.${pkgs.system}.vscode-marketplace; [
                  jnoortheen.nix-ide
                  kamadorueda.alejandra
                  kokakiwi.vscode-just
                  rust-lang.rust-analyzer
                  tamasfe.even-better-toml
                  fill-labs.dependi
                  guyutongxue.lalrpop-syntax-highlight
                ];
              })
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      }
    );
}
