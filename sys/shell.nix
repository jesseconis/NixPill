{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.htop   # Installing htop as an example system package
  ];

  shellHook = ''
    echo "htop is available in this shell!"
  '';
}

