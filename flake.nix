{
  description = "Flakebox Project template";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    flakebox.url = "github:rustshop/flakebox?rev=5e9ce550fb989f1311547ee09301315cc311ba3b";
  };

  outputs =
    {
      self,
      flake-utils,
      flakebox,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        projectName = "flakebox-project";

        flakeboxLib = flakebox.lib.${system} {
          config = {
            github.ci.buildOutputs = [ ".#ci.${projectName}" ];
            github.ci.flakeSelfCheck.enable = false;
          };
        };

        buildPaths = [
          "README.md"
          "Cargo.toml"
          "Cargo.lock"
          "src"
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
            craneLib = (
              craneLib'.overrideArgs {
                pname = projectName;
                src = buildSrc;
                nativeBuildInputs = [ ];
              }
            );
          in
          {
            ${projectName} = craneLib.buildPackage { };
          }
        );
      in
      {
        packages.default = multiBuild.${projectName};

        legacyPackages = multiBuild;

        devShells = flakeboxLib.mkShells { };
      }
    );
}
