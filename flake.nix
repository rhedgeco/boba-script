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
                vscodeExtensions = with pkgs.vscode-extensions;
                  [
                    jnoortheen.nix-ide
                    kamadorueda.alejandra
                    rust-lang.rust-analyzer
                    tamasfe.even-better-toml
                    fill-labs.dependi
                    vadimcn.vscode-lldb
                  ]
                  ++ (with nix-vscode-extensions.extensions.${pkgs.system}.vscode-marketplace; [
                    kokakiwi.vscode-just
                    guyutongxue.lalrpop-syntax-highlight
                  ]);
              })
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      }
    );
}
