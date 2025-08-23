{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  name = "htmx-seaorm";
  packages = with pkgs; [
    just
    podman-compose
    sqlx-cli
    postgresql_16_jit
    openssl
    pkg-config
    massif-visualizer
    valgrind
    gnuplot
    python312Packages.tkinter
    openblas
    libz
    cmake
    sea-orm-cli
    cargo-watch
  ];
  buildInputs = with pkgs; [
    stdenv.cc.cc.lib
  ];
  shellHook = ''
    export LD_LIBRARY_PATH=${pkgs.zlib.out}/lib:${pkgs.stdenv.cc.cc.lib}/lib/:$LD_LIBRARY_PATH
  '';
}
