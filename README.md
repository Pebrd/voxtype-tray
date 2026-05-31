# voxtype-tray

System tray indicator for [voxtype](https://github.com/pebrd/voxtype) — muestra el estado de la grabación (idle, recording, transcribing, error) con un indicador circular colorido en el área de notificación.

![idle](https://img.shields.io/badge/idle-green) ![recording](https://img.shields.io/badge/recording-red) ![transcribing](https://img.shields.io/badge/transcribing-orange) ![error](https://img.shields.io/badge/error-dark_red) ![inactive](https://img.shields.io/badge/inactive-gray)

## Instalación

```bash
curl -sSL https://raw.githubusercontent.com/pebrd/voxtype-tray/main/install | bash
```

Requiere Rust ≥ 1.70 y `libdbus-1-dev` / `libdbus-1-dev` / `dbus-devel` según tu distro.

### Prerrequisitos

| Distro | Comando |
|--------|---------|
| Debian / Ubuntu | `sudo apt install libdbus-1-dev pkg-config` |
| Arch Linux | `sudo pacman -S dbus pkgconf` |
| Fedora | `sudo dnf install dbus-devel pkgconfig` |

## Uso

El servicio se instala como un systemd user unit y arranca automáticamente al iniciar sesión gráfica.

```bash
# Ver logs
journalctl --user -u voxtype-tray -f

# Detener
systemctl --user stop voxtype-tray

# Desinstalar
systemctl --user disable --now voxtype-tray
rm ~/.local/bin/voxtype-tray
```

## Estados

| Estado | Color | Tooltip |
|--------|-------|---------|
| idle | 🟢 verde | Voxtype — listo |
| recording | 🔴 rojo | Voxtype — grabando... |
| transcribing | 🟠 ámbar | Voxtype — transcribiendo... |
| error | 🔴 rojo oscuro | Voxtype — error |
| inactive | ⚫ gris | (sin conexión) |

## Cómo funciona

`voxtype-tray` ejecuta `voxtype status --follow --format json` y reacciona a cada línea JSON actualizando el icono del system tray vía [`ksni`](https://crates.io/crates/ksni) (protocolo StatusNotifierItem).

## Build manual

```bash
git clone https://github.com/pebrd/voxtype-tray
cd voxtype-tray
cargo build --release
cp target/release/voxtype-tray ~/.local/bin/
```

## Licencia

MIT
