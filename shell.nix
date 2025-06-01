{
  pkgs ? import <nixpkgs> { },
}:
let
  rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
in
pkgs.mkShell {
#  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

  nativeBuildInputs = [
#    pkgs.clang
    pkgs.pkg-config
    rustToolchain
  ];

  # run time dependencies
  buildInputs = [ ];
}
