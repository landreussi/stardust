{
  description = "Rust development environment with pinned toolchain via rustup";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system};
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [ cargo rustup pkg-config openssl ];

          shellHook = ''
            rustup component add rust-src rust-analyzer
          '';
        };
      });
}
