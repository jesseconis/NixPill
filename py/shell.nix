{ pkgs ? import <nixpkgs> {} }:

let
  python = pkgs.python312;

in
pkgs.mkShell {
  buildInputs = [
    python
    python.pkgs.pydantic
    python.pkgs.pip
    python.pkgs.fastapi
    pkgs.uv
  ];

  shellHook = ''
    echo "Python environment with pydantic from nixpkgs and robyn installed via pip."
  '';
}
