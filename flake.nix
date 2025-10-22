{
  inputs = {
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    devenv.inputs.nixpkgs.follows = "nixpkgs";
    serena.url = "github:oraios/serena";
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs =
    {
      self,
      nixpkgs,
      devenv,
      systems,
      serena,
      ...
    }@inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      packages = forEachSystem (system: {
        devenv-up = self.devShells.${system}.default.config.procfileScript;
        devenv-test = self.devShells.${system}.default.config.test;
      });

      devShells = forEachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [
              {

                scripts.claude.exec = ''
                  bunx @anthropic-ai/claude-code "$@";
                '';
                # scripts.serena.exec = ''
                #   uvx --from git+https://github.com/oraios/serena serena start-mcp-server
                # '';
                packages = (
                  (with pkgs; [
                    uv
                    bun
                    cargo-insta
                  ])
                  ++ ([
                    serena.packages.${system}.serena
                  ])
                );

                languages.rust = {
                  enable = true;
                  channel = "nixpkgs";
                  components = [
                    "rustc"
                    "cargo"
                    "clippy"
                    "rustfmt"
                    "rust-analyzer"
                  ];
                };

                # enterShell = '''';
                # processes.hello.exec = "hello";
              }
            ];
          };
        }
      );
    };
}
