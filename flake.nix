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
            if [ -z "''${XDG_RUNTIME_DIR:-}" ]; then
              export XDG_RUNTIME_DIR="/tmp/xdg-runtime-$UID"
              mkdir -p "$XDG_RUNTIME_DIR"
              chmod 700 "$XDG_RUNTIME_DIR"
            fi

            # `nix develop` points TMPDIR at an ephemeral NIX_BUILD_TOP. Cargo and
            # cmake need a stable writable scratch dir for the whole session.
            export TMPDIR="''${XDG_RUNTIME_DIR:-/tmp}"
            if [ ! -d "$TMPDIR" ] || [ ! -w "$TMPDIR" ]; then
              export TMPDIR="/tmp"
            fi
            export TMP="$TMPDIR"
            export TEMP="$TMPDIR"
            export TEMPDIR="$TMPDIR"

            export LD_LIBRARY_PATH="${libraryPath}:''${LD_LIBRARY_PATH:-}"
            export LIBRARY_PATH="${libraryPath}:''${LIBRARY_PATH:-}"
            export XDG_DATA_DIRS="${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:''${XDG_DATA_DIRS:-}"
            export GIO_MODULE_DIR="${pkgs.glib-networking}/lib/gio/modules"

            echo "volt nix shell ready (rust $(rustc --version))"
            echo "  cargo xtask ci"
            echo "  cargo run -p volt -- --shell-hidden"

            # `nix develop` always starts bashInteractive and overwrites SHELL.
            # Re-exec the user's login shell for interactive sessions only.
            # Skip: direnv (already in user's shell), `nix develop -c` (non-interactive),
            # and NIX_SHELL_PRESERVE_BASH=1 / VOLT_DEV_SHELL overrides.
            if [ -n "''${VOLT_DEV_SHELL:-}" ]; then
              user_shell="''${VOLT_DEV_SHELL}"
            else
              user_shell="$(getent passwd "$(id -un)" 2>/dev/null | cut -d: -f7 || true)"
              if [ -z "$user_shell" ] && [ -r /etc/passwd ]; then
                user_shell="$(awk -F: -v u="$(id -un)" '$1 == u { print $7; exit }' /etc/passwd)"
              fi
            fi
            if [ -z "''${DIRENV_DIR:-}" ] && [ -z "''${DIRENV_IN_ENVRC:-}" ] \
              && [ "''${NIX_SHELL_PRESERVE_BASH:-}" != 1 ] \
              && [[ $- == *i* ]] \
              && [ -n "$user_shell" ] && [ -x "$user_shell" ]; then
              case "$(basename -- "$user_shell")" in
                bash|sh) ;;
                zsh)
                  # User ~/.zshrc / /etc/zshenv often reset PATH and drop the nix
                  # stdenv (→ missing headers, empty .pc files, broken cmake).
                  # Shim ZDOTDIR: load user rc, then restore toolchain vars.
                  _volt_zdot="$(mktemp -d "$TMPDIR/volt-zdot.XXXXXX")"
                  export VOLT_USER_ZDOTDIR="''${ZDOTDIR:-$HOME}"
                  export VOLT_KEEP_PATH="$PATH"
                  export VOLT_KEEP_PKG_CONFIG_PATH="''${PKG_CONFIG_PATH:-}"
                  export VOLT_KEEP_LD_LIBRARY_PATH="''${LD_LIBRARY_PATH:-}"
                  export VOLT_KEEP_LIBRARY_PATH="''${LIBRARY_PATH:-}"
                  export VOLT_KEEP_NIX_CFLAGS_COMPILE="''${NIX_CFLAGS_COMPILE:-}"
                  export VOLT_KEEP_NIX_CFLAGS_LINK="''${NIX_CFLAGS_LINK:-}"
                  export VOLT_KEEP_NIX_LDFLAGS="''${NIX_LDFLAGS:-}"
                  export VOLT_KEEP_XDG_DATA_DIRS="''${XDG_DATA_DIRS:-}"
                  export VOLT_KEEP_GIO_MODULE_DIR="''${GIO_MODULE_DIR:-}"
                  cat > "$_volt_zdot/.zshrc" <<'ZSHRC'
if [ -f "$VOLT_USER_ZDOTDIR/.zshenv" ]; then
  source "$VOLT_USER_ZDOTDIR/.zshenv"
fi
if [ -f "$VOLT_USER_ZDOTDIR/.zshrc" ]; then
  source "$VOLT_USER_ZDOTDIR/.zshrc"
fi
export PATH="$VOLT_KEEP_PATH''${PATH:+:$PATH}"
export PKG_CONFIG_PATH="$VOLT_KEEP_PKG_CONFIG_PATH''${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
export LD_LIBRARY_PATH="$VOLT_KEEP_LD_LIBRARY_PATH''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export LIBRARY_PATH="$VOLT_KEEP_LIBRARY_PATH''${LIBRARY_PATH:+:$LIBRARY_PATH}"
export XDG_DATA_DIRS="$VOLT_KEEP_XDG_DATA_DIRS''${XDG_DATA_DIRS:+:$XDG_DATA_DIRS}"
[ -n "$VOLT_KEEP_NIX_CFLAGS_COMPILE" ] && export NIX_CFLAGS_COMPILE="$VOLT_KEEP_NIX_CFLAGS_COMPILE"
[ -n "$VOLT_KEEP_NIX_CFLAGS_LINK" ] && export NIX_CFLAGS_LINK="$VOLT_KEEP_NIX_CFLAGS_LINK"
[ -n "$VOLT_KEEP_NIX_LDFLAGS" ] && export NIX_LDFLAGS="$VOLT_KEEP_NIX_LDFLAGS"
[ -n "$VOLT_KEEP_GIO_MODULE_DIR" ] && export GIO_MODULE_DIR="$VOLT_KEEP_GIO_MODULE_DIR"
ZSHRC
                  export ZDOTDIR="$_volt_zdot"
                  export SHELL="$user_shell"
                  exec "$user_shell" --no-globalrcs
                  ;;
                *)
                  export SHELL="$user_shell"
                  exec "$user_shell"
                  ;;
              esac
            fi
          '';
        };

        # Convenience alias: `nix develop .#volt`
        devShells.volt = self.devShells.${system}.default;

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
