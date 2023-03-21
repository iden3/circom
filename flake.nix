{ inputs =
    { cargo2nix.url = "github:cargo2nix/cargo2nix";
      nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
      rust-overlay.url = "github:oxalica/rust-overlay";
      utils.url = "github:ursi/flake-utils/8";
    };

  outputs = { cargo2nix, rust-overlay, utils, ... }@inputs:
    with builtins;
    utils.apply-systems
      { inherit inputs;

        overlays =
          [ cargo2nix.overlays.default
            rust-overlay.overlays.default
          ];

        systems = utils.default-systems ++ ["aarch64-darwin"];
      }
      ({ cargo2nix, pkgs, system, ... }:
         let
           rustChannel = "1.67.1";
           rustPkgs =
             pkgs.rustBuilder.makePackageSet
               { rustChannel = rustChannel;
                 packageFun = import ./Cargo.nix;
               };
         in
         { inherit rustPkgs;
           defaultPackage = rustPkgs.workspace.circom {};

           devShell =
             let
               rust-toolchain =
                 (pkgs.formats.toml {}).generate "rust-toolchain.toml"
                   { toolchain =
                       { channel = rustChannel;

                         components =
                           [ "rustc"
                             "rust-src"
                             "cargo"
                             "clippy"
                             "rust-docs"
                           ];
                       };
                   };
             in
             rustPkgs.workspaceShell {
               nativeBuildInputs = with pkgs; [ rust-analyzer rustup ];
               shellHook =
                   ''
                   cp --no-preserve=mode ${rust-toolchain} rust-toolchain.toml

                   export RUST_SRC_PATH=~/.rustup/toolchains/${rustChannel}-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/
                   '';
               };
         }
      );
}
