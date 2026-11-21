{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    pkg-config
    glib
    gtk4
    libadwaita
    graphene
    gdk-pixbuf
    pango
    cairo
    harfbuzz
  ];

  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
}
