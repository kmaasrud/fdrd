{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    crane,
    fenix,
    ...
  }: let
    eachSystem = systems: f: let
      # Merge together the outputs for all systems.
      op = attrs: system: let
        ret = f system;
        op = attrs: key:
          attrs
          // {
            ${key} =
              (attrs.${key} or {})
              // {${system} = ret.${key};};
          };
      in
        builtins.foldl' op attrs (builtins.attrNames ret);
    in
      builtins.foldl' op {} systems;
    systems = ["x86_64-linux" "aarch64-linux"];
    forAllSystems = eachSystem systems;
  in
    forAllSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
      };

      crossPkgs = import nixpkgs {
        inherit system;
        crossSystem = "armv6l-linux";
      };

      toolchain = with fenix.packages.${system};
        combine [
          stable.rustc
          stable.cargo
          stable.clippy
          stable.rustfmt
          targets.arm-unknown-linux-musleabihf.stable.rust-std
        ];

      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
    in rec {
      packages.default = craneLib.buildPackage rec {
        src = pkgs.lib.cleanSourceWith {src = craneLib.path ./.;};
      };

      apps.default = {
        drv = packages.default;
      };

      devShells.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [rust-analyzer toolchain];
        # CARGO_TARGET_ARM_UNKNOWN_LINUX_MUSLEABIHF_LINKER = "${pkgs.stdenv.cc.targetPrefix}cc";
      };

      devShells.pizero = crossPkgs.mkShell {
        nativeBuildInputs = with crossPkgs; [toolchain];
        CARGO_TARGET_ARM_UNKNOWN_LINUX_MUSLEABIHF_LINKER = "${crossPkgs.stdenv.cc.targetPrefix}cc";
      };
    });
}
