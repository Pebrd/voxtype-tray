#!/bin/bash
set -e

BINARY=$(grep '^name' Cargo.toml | head -1 | sed 's/.*= *"\(.*\)"/\1/')
SERVICE="voxtype-tray"
INSTALL_DIR="$HOME/.local/bin"
SERVICE_DIR="$HOME/.config/systemd/user"

echo "==> Verificando dependencias de sistema (libdbus)..."
if ! pkg-config --exists dbus-1 2>/dev/null; then
    echo "Falta libdbus-dev:"
    echo "  Debian/Ubuntu: sudo apt install libdbus-1-dev pkg-config"
    echo "  Arch:          sudo pacman -S dbus pkgconf"
    echo "  Fedora:        sudo dnf install dbus-devel pkgconfig"
    exit 1
fi

echo "==> Build release..."
cargo build --release

echo "==> Instalando $BINARY..."
mkdir -p "$INSTALL_DIR"

# Detener servicio si está corriendo antes de reemplazar el binario
if systemctl --user is-active --quiet "$SERVICE" 2>/dev/null; then
    echo "==> Deteniendo $SERVICE..."
    systemctl --user stop "$SERVICE"
fi

cp "target/release/$BINARY" "$INSTALL_DIR/$BINARY"
chmod +x "$INSTALL_DIR/$BINARY"

echo "==> Instalando servicio systemd..."
mkdir -p "$SERVICE_DIR"
cat > "$SERVICE_DIR/$SERVICE.service" << SVCEOF
[Unit]
Description=Voxtype system tray indicator
After=graphical-session.target
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart=$INSTALL_DIR/$BINARY
Restart=on-failure
RestartSec=3

[Install]
WantedBy=graphical-session.target
SVCEOF

systemctl --user daemon-reload
systemctl --user enable --now "$SERVICE"

echo ""
echo "✓ Listo."
echo "  Logs:        journalctl --user -u $SERVICE -f"
echo "  Detener:     systemctl --user stop $SERVICE"
echo "  Desinstalar: systemctl --user disable --now $SERVICE && rm $INSTALL_DIR/$BINARY"
