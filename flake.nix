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
          buildInputs = with pkgs; [
            cargo
            cargo-watch
            rustup
            pkg-config
            openssl
            alsa-lib
            libxkbcommon
            xorg.libX11

            # Vulkan
            vulkan-loader
            vulkan-headers
            vulkan-tools
          ];

          env = {
            LD_LIBRARY_PATH =
              "${pkgs.libxkbcommon}/lib:${pkgs.vulkan-loader}/lib";
            RUST_BACKTRACE = "1";
          };

          shellHook = ''
            rustup component add rust-src rust-analyzer
            echo "${pkgs.vulkan-loader}"
          '';
        };
      });
}
