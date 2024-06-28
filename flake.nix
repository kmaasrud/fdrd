{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = {nixpkgs, ...}: let
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
      pkgs = import nixpkgs {inherit system;};
    in {
      devShells.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          lightningcss
          minify
          nodePackages.uglify-js
          python3
        ];
      };
    });
}
