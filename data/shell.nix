{
  pkgs ? import <nixpkgs> { },
}:
let
  rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
in
pkgs.mkShell {
  nativeBuildInputs = [
    pkgs.pkg-config

    rustToolchain
  ];

  # run time dependencies (dynamic linking)
  buildInputs = [
    pkgs.libressl
  ];
}
