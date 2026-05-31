# voxtype-tray

System tray indicator for [voxtype](https://github.com/pebrd/voxtype) — muestra el estado de la grabación (idle, recording, transcribing, error) con un indicador circular colorido en el área de notificación.

![idle](https://img.shields.io/badge/idle-sage-brightgreen) ![recording](https://img.shields.io/badge/recording-danger-red) ![transcribing](https://img.shields.io/badge/transcribing-amber-orange) ![error](https://img.shields.io/badge/error-brick-darkred) ![inactive](https://img.shields.io/badge/inactive-disabled-lightgray)

Colores del sistema [Nox](https://github.com/pebrd/nox).

## Instalación rápida

```bash
curl -sSL https://raw.githubusercontent.com/pebrd/voxtype-tray/main/install | bash
```

Baja el binario pre-compilado desde GitHub Releases. Si no hay release, compila desde fuente.

## Descarga manual

Bajá el archivo de la [última release](https://github.com/pebrd/voxtype-tray/releases/latest):

```bash
# x86_64
curl -sL https://github.com/pebrd/voxtype-tray/releases/latest/download/voxtype-tray-x86_64-unknown-linux-gnu.tar.gz \
  | tar xz
chmod +x voxtype-tray
mv voxtype-tray ~/.local/bin/
```

```bash
# ARM64 (aarch64)
curl -sL https://github.com/pebrd/voxtype-tray/releases/latest/download/voxtype-tray-aarch64-unknown-linux-gnu.tar.gz \
  | tar xz
chmod +x voxtype-tray
mv voxtype-tray ~/.local/bin/
```

### Servicio systemd

```ini
# ~/.config/systemd/user/voxtype-tray.service
[Unit]
Description=Voxtype system tray indicator
After=graphical-session.target
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.local/bin/voxtype-tray
Restart=on-failure
RestartSec=3

[Install]
WantedBy=graphical-session.target
```

```bash
systemctl --user daemon-reload
systemctl --user enable --now voxtype-tray
```

## Compilar desde fuente

Requiere Rust ≥ 1.70 y `libdbus-1-dev` / `dbus-devel`.

```bash
git clone https://github.com/pebrd/voxtype-tray
cd voxtype-tray
cargo build --release
cp target/release/voxtype-tray ~/.local/bin/
```

### Prerrequisitos por distro

| Distro | Comando |
|--------|---------|
| Debian / Ubuntu | `sudo apt install libdbus-1-dev pkg-config` |
| Arch Linux | `sudo pacman -S dbus pkgconf` |
| Fedora | `sudo dnf install dbus-devel pkgconfig` |

## Estados

| Estado | Color | Tooltip |
|--------|-------|---------|
| idle | 🟢 `#7EAB8A` sage | Voxtype — listo |
| recording | 🔴 `#C87070` danger | Voxtype — grabando... |
| transcribing | 🟠 `#C4A96A` amber | Voxtype — transcribiendo... |
| error | 🔴 `#A87070` brick | Voxtype — error |
| inactive | ⚫ `#808080` disabled | (sin conexión) |

## Cómo funciona

`voxtype-tray` ejecuta `voxtype status --follow --format json` y reacciona a cada línea JSON actualizando el icono del system tray vía [`ksni`](https://crates.io/crates/ksni) (protocolo StatusNotifierItem).

## Release

Los tags `v*` disparan una GitHub Action que compila y sube el binario a la release automáticamente:

```bash
git tag v0.1.0
git push origin v0.1.0
```

## Licencia

MIT
