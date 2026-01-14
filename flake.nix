{
  description = "fzf-keys - keybinding discovery tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustc
            cargo
            rustfmt
            clippy

            # Python and kitty
            python313
            kitty

            # Build dependencies for PyO3
            pkg-config

            # Nice to have
            rust-analyzer
          ];

          # Set environment variables for PyO3
          PYTHON_SYS_EXECUTABLE = "${pkgs.python313}/bin/python";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [ pkgs.python313 ]}";
          PYTHONPATH = "${pkgs.kitty}/lib/kitty";

          shellHook = ''
            echo "fzf-keys development environment"
            echo "Rust version: $(rustc --version)"
            echo "Python version: $(python --version)"
            echo "Kitty available: $(python -c 'import kitty.debug_config; print("yes")' 2>/dev/null || echo 'no')"
          '';
        };
      }
    );
}
