let
  pkgs = import (fetchTarball("channel:nixpkgs-unstable")) {};
in pkgs.mkShell {
  # necessary stuff for building the app
  buildInputs = with pkgs;[
    cargo
    rustc
    cargo-tauri
    rust-analyzer
    glib.dev
    pango.dev
    cairo.dev
    glib
    gdk-pixbuf.dev
    pkg-config
    gdk-pixbuf-xlib.dev
    gtk3.dev
    harfbuzz.dev
    atk.dev
    libsoup_3.dev
    webkitgtk_4_1.dev
    libva-utils
    vdpauinfo
    vulkan-tools
    vulkan-validation-layers
    libvdpau-va-gl
    egl-wayland
    wgpu-utils
    mesa
    libglvnd
    libGL
  ];
}
