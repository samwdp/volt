{
  description = "Volt editor development environment (Linux / WSL2)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Workspace requires stable >= 1.91 (edition 2024). Prefer latest stable
        # from the overlay so rustc/clippy/rustfmt stay in sync with CI.
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustfmt"
          ];
        };

        # SDL3 is compiled from source via the sdl3 crate
        # (build-from-source-static). These match the apt packages in CI.
        sdlBuildInputs = with pkgs; [
          alsa-lib
          dbus
          fribidi
          libdecor
          libdrm
          libGL
          libglvnd
          libjack2
          libthai
          libxkbcommon
          mesa
          pipewire
          pulseaudio
          sndio
          udev
          vulkan-loader
          wayland
          wayland-protocols
          libx11
          libxcursor
          libxext
          libxfixes
          libxi
          libxrandr
          libxrender
          libxscrnsaver
          libxtst
        ];

        # wry / gtk-rs / WebKitGTK 4.1 (embedded browser host)
        webkitBuildInputs = with pkgs; [
          at-spi2-atk
          cairo
          gdk-pixbuf
          glib
          glib-networking
          gobject-introspection
          gtk3
          harfbuzz
          librsvg
          libsoup_3
          openssl
          pango
          webkitgtk_4_1
        ];

        # SDL_ttf (FreeType) + misc link deps
        otherBuildInputs = with pkgs; [
          freetype
          zlib
        ];

        allBuildInputs = sdlBuildInputs ++ webkitBuildInputs ++ otherBuildInputs;

        libraryPath = pkgs.lib.makeLibraryPath (
          allBuildInputs ++ [ pkgs.stdenv.cc.cc.lib ]
        );
      in
      {
        devShells.default = pkgs.mkShell {
          name = "volt";

          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
            cmake
            ninja
            git
            gcc
            gnumake
            python3
            # GSettings / GTK schema discovery for wry/WebKit at runtime
            wrapGAppsHook3
          ];

          buildInputs = allBuildInputs;

          packages = with pkgs; [
            gdb
            lldb
          ];

          # rust-analyzer needs std sources from the overlay toolchain.
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          # Soft-GL is common under WSL2 / remote desktops without a GPU.
          LIBGL_ALWAYS_SOFTWARE = "1";

          # WebKit sometimes blanks out under Nix without this.
          WEBKIT_DISABLE_COMPOSITING_MODE = "1";

          shellHook = ''
            export LD_LIBRARY_PATH="${libraryPath}:''${LD_LIBRARY_PATH:-}"
            export LIBRARY_PATH="${libraryPath}:''${LIBRARY_PATH:-}"
            export XDG_DATA_DIRS="${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:''${XDG_DATA_DIRS:-}"
            export GIO_MODULE_DIR="${pkgs.glib-networking}/lib/gio/modules"

            if [ -z "''${XDG_RUNTIME_DIR:-}" ]; then
              export XDG_RUNTIME_DIR="/tmp/xdg-runtime-$UID"
              mkdir -p "$XDG_RUNTIME_DIR"
              chmod 700 "$XDG_RUNTIME_DIR"
            fi

            echo "volt nix shell ready (rust $(rustc --version))"
            echo "  cargo xtask ci"
            echo "  cargo run -p volt -- --shell-hidden"
          '';
        };

        # Convenience alias: `nix develop .#volt`
        devShells.volt = self.devShells.${system}.default;

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
