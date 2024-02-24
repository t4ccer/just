{
  description = "justshow";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    pre-commit-hooks-nix = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nixpkgs-stable.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
    };
  };
  outputs = inputs @ {self, ...}:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.pre-commit-hooks-nix.flakeModule
      ];

      # `nix flake show --impure` hack
      systems =
        if builtins.hasAttr "currentSystem" builtins
        then [builtins.currentSystem]
        else inputs.nixpkgs.lib.systems.flakeExposed;

      flake = {
        nixosConfigurations = {
          wm = self.inputs.nixpkgs.lib.nixosSystem rec {
            system = "x86_64-linux";
            modules = [
              (import ./nix/wm-vm.nix {
                inherit (self.packages.${system}) justwindows;
              })
            ];
          };
        };
      };

      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        lib,
        system,
        ...
      }: let
        rustToolchain = pkgs.rust-bin.fromRustupToolchain {
          channel = "stable";
          components = ["rust-analyzer" "rust-src" "rustfmt" "rustc" "cargo"];
          targets = ["x86_64-unknown-linux-gnu" "x86_64-unknown-linux-musl"];
        };
      in {
        _module.args.pkgs = import self.inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.rust-overlay.overlays.rust-overlay
          ];
        };

        pre-commit.settings = {
          src = ./.;
          hooks = {
            alejandra.enable = true;
            rustfmt.enable = true;
            clippy.enable = true;
          };
          tools = {
            rustfmt = lib.mkForce rustToolchain;
            clippy = lib.mkForce rustToolchain;
          };
        };

        apps = {
          vm-wm.program = "${self.nixosConfigurations.wm.config.system.build.vm}/bin/run-nixos-vm";
        };

        packages = {
          justwindows = pkgs.rustPlatform.buildRustPackage {
            name = "justwindows";

            src = builtins.filterSource (path: type: !(lib.hasSuffix ".nix" path)) ./.;
            buildAndTestSubdir = "crates/justwindows";

            cargoLock.lockFile = ./Cargo.lock;
          };
        };

        devShells.default = pkgs.mkShell {
          shellHook = ''
            ${config.pre-commit.installationScript}
            PATH=$PATH:$(pwd)/target/release
          '';

          env = {
            LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib";
          };

          nativeBuildInputs = [
            pkgs.alejandra
            pkgs.cargo-expand
            pkgs.cargo-flamegraph
            pkgs.cargo-leptos
            pkgs.cargo-machete
            pkgs.cargo-modules
            pkgs.cargo-nextest
            pkgs.cargo-semver-checks
            pkgs.cargo-tarpaulin
            pkgs.cargo-udeps
            pkgs.fd
            pkgs.heaptrack
            pkgs.hyperfine
            pkgs.linuxKernel.packages.linux_6_1.perf
            rustToolchain
            pkgs.xlsfonts
          ];
        };
        formatter = pkgs.alejandra;
      };
    };
}
