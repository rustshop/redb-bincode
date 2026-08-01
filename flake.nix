{
  description = "Typed bincode wrapper for redb";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    flake-utils.url = "github:numtide/flake-utils";
    flakebox = {
      url = "github:rustshop/flakebox?rev=cf89db7a3ac6b1431693d17276225ba352e48a5c";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      flakebox,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        projectName = "redb-bincode";
        cargoCrap = pkgs.callPackage ./nix/pkgs/cargo-crap.nix { };

        flakeboxLib = flakebox.lib.mkLib pkgs {
          config = {
            github.ci.enable = true;
            github.ci.buildOutputs = [ ".#ci.${projectName}" ];
            github.ci.flakeSelfCheck.enable = false;
            just.rules.watch.enable = false;
            linker.wild.enable = false;
            linker.mold.enable = true;
            toolchain.components = [
              "rustc"
              "cargo"
              "clippy"
              "rust-analyzer"
              "rust-src"
              "rustfmt"
              "llvm-tools"
            ];
          };
        };

        buildPaths = [
          ".rustfmt.toml"
          "README.md"
          "Cargo.toml"
          ".cargo-crap.toml"
          "Cargo.lock"
          "src"
          "tests"
          "examples"
          "nix/cargo-crap-baseline.json"
        ];

        buildSrc = flakeboxLib.filterSubPaths {
          root = builtins.path {
            name = projectName;
            path = ./.;
          };
          paths = buildPaths;
        };

        multiBuild = (flakeboxLib.craneMultiBuild { }) (
          craneLib':
          let
            craneLib = craneLib'.overrideArgs {
              pname = projectName;
              src = buildSrc;
              nativeBuildInputs = [ ];
              env.RUSTDOCFLAGS = "-D warnings";
            };
          in
          rec {
            workspaceDeps = craneLib.buildWorkspaceDepsOnly { };

            workspace = craneLib.buildWorkspace {
              cargoArtifacts = workspaceDeps;
            };

            tests = craneLib.cargoNextest {
              cargoArtifacts = workspace;
              doInstallCargoArtifacts = false;
              cargoNextestExtraArgs = "--workspace --show-progress none";
            };

            workspaceDepsCcov = craneLib.buildDepsOnly {
              pname = "${projectName}-workspace-ccov";
              buildPhaseCargoCommand = ''
                source <(cargo llvm-cov show-env --export-prefix)
                cargo build --locked --workspace --all-targets --profile $CARGO_PROFILE
              '';
              cargoBuildCommand = "dontuse";
              nativeBuildInputs = [ pkgs.cargo-llvm-cov ];
              doCheck = false;
            };

            workspaceCcov = craneLib.buildWorkspace {
              pname = "${projectName}-workspace-ccov";
              cargoArtifacts = workspaceDepsCcov;
              buildPhaseCargoCommand = ''
                source <(cargo llvm-cov show-env --export-prefix)
                cargo build --locked --workspace --all-targets --profile $CARGO_PROFILE
              '';
              nativeBuildInputs = [ pkgs.cargo-llvm-cov ];
              doCheck = false;
            };

            testsCcov = craneLib.mkCargoDerivation {
              pname = "${projectName}-tests-ccov";
              cargoArtifacts = workspaceCcov;
              buildPhaseCargoCommand = ''
                source <(cargo llvm-cov show-env --export-prefix)
                cargo nextest run --locked --workspace --all-targets --cargo-profile $CARGO_PROFILE --show-progress none
                mkdir -p $out
                cargo llvm-cov report --profile $CARGO_PROFILE --lcov --output-path $out/lcov.info
                test -s $out/lcov.info
              '';
              doInstallCargoArtifacts = false;
              nativeBuildInputs = [
                pkgs.cargo-llvm-cov
                pkgs.cargo-nextest
              ];
              doCheck = false;
            };

            # Regenerate nix/cargo-crap-baseline.json from this derivation only
            # after intentional CRAP-score changes land on the mainline.
            crapBaseline = craneLib.mkCargoDerivation {
              pname = "${projectName}-cargo-crap-ccov-baseline";
              cargoArtifacts = null;
              buildPhaseCargoCommand = ''
                test -s ${testsCcov}/lcov.info
                mkdir -p $out
                ${cargoCrap}/bin/cargo-crap --workspace --lcov ${testsCcov}/lcov.info \
                  --format json --output $out/cargo-crap-baseline.json
              '';
              doInstallCargoArtifacts = false;
              nativeBuildInputs = [ cargoCrap ];
              doCheck = false;
            };

            crapReport = craneLib.mkCargoDerivation {
              pname = "${projectName}-cargo-crap-ccov-report";
              cargoArtifacts = null;
              buildPhaseCargoCommand = ''
                test -s ${testsCcov}/lcov.info
                mkdir -p $out
                ${cargoCrap}/bin/cargo-crap --workspace --lcov ${testsCcov}/lcov.info \
                  --threshold 30 --top 100 --min 50 --format markdown \
                  --output $out/cargo-crap.md
                cp ${testsCcov}/lcov.info $out/lcov.info
              '';
              doInstallCargoArtifacts = false;
              nativeBuildInputs = [ cargoCrap ];
              doCheck = false;
            };

            crapRegression = craneLib.mkCargoDerivation {
              pname = "${projectName}-cargo-crap-ccov-regression";
              cargoArtifacts = null;
              buildPhaseCargoCommand = ''
                test -s ${testsCcov}/lcov.info
                ${cargoCrap}/bin/cargo-crap --workspace --lcov ${testsCcov}/lcov.info \
                  --baseline ${./nix/cargo-crap-baseline.json} \
                  --threshold 1000 --min 1000 --format github --fail-regression
                mkdir -p $out
                cp ${testsCcov}/lcov.info $out/lcov.info
              '';
              doInstallCargoArtifacts = false;
              nativeBuildInputs = [ cargoCrap ];
              doCheck = false;
            };

            crapAbsolute = craneLib.mkCargoDerivation {
              pname = "${projectName}-cargo-crap-ccov-absolute";
              cargoArtifacts = null;
              buildPhaseCargoCommand = ''
                test -s ${testsCcov}/lcov.info
                ${cargoCrap}/bin/cargo-crap --workspace --lcov ${testsCcov}/lcov.info \
                  --min 100 --format github --fail-above
                mkdir -p $out
                cp ${testsCcov}/lcov.info $out/lcov.info
              '';
              doInstallCargoArtifacts = false;
              nativeBuildInputs = [ cargoCrap ];
              doCheck = false;
            };

            crap = pkgs.runCommand "${projectName}-cargo-crap-ccov" { } ''
              mkdir -p $out
              ln -s ${crapRegression} $out/regression
              ln -s ${crapAbsolute} $out/absolute
              cp ${crapRegression}/lcov.info $out/lcov.info
            '';

            clippy = craneLib.cargoClippy {
              cargoArtifacts = workspaceDeps;
              doInstallCargoArtifacts = false;
              cargoClippyExtraArgs = "-- -D warnings";
            };

            cargoFmt = craneLib.cargoFmt { };

            ${projectName} = craneLib.buildPackage {
              cargoArtifacts = workspaceDeps;
            };
          }
        );

        treefmt =
          pkgs.runCommand "treefmt-check"
            {
              nativeBuildInputs = [
                pkgs.treefmt
                pkgs.nixfmt
                pkgs.rustfmt
                pkgs.taplo
              ];
              src = self;
            }
            ''
              cp -r $src work && chmod -R u+w work
              cd work
              treefmt --ci
              touch $out
            '';
      in
      {
        packages = {
          default = multiBuild.${projectName};
          ${projectName} = multiBuild.${projectName};
          inherit treefmt;
        };

        ci = {
          inherit (multiBuild)
            workspace
            tests
            clippy
            cargoFmt
            workspaceCcov
            testsCcov
            crapBaseline
            crapReport
            crapRegression
            crapAbsolute
            crap
            ;
          ${projectName} = multiBuild.${projectName};
        };

        legacyPackages = multiBuild;

        devShells = flakeboxLib.mkShells {
          packages = [
            cargoCrap
            pkgs.cargo-nextest
            pkgs.taplo
          ];
        };
      }
    );
}
