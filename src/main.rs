use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender};
use std::thread;

use ksni::blocking::TrayMethods;
use serde::Deserialize;

// ─── Colores por estado (ARGB) ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum VoxtypeState {
    Idle,
    Recording,
    Transcribing,
    Error,
    Inactive,
}

impl VoxtypeState {
    fn color(&self) -> u32 {
        match self {
            Self::Idle         => 0xFF4CAF50, // verde
            Self::Recording    => 0xFFF44336, // rojo
            Self::Transcribing => 0xFFFF9800, // ámbar
            Self::Error        => 0xFF880000, // rojo oscuro
            Self::Inactive     => 0xFF9E9E9E, // gris
        }
    }

    fn tooltip(&self) -> &'static str {
        match self {
            Self::Idle         => "Voxtype — listo",
            Self::Recording    => "Voxtype — grabando...",
            Self::Transcribing => "Voxtype — transcribiendo...",
            Self::Error        => "Voxtype — error",
            Self::Inactive     => "Voxtype — inactivo",
        }
    }
}

impl From<&str> for VoxtypeState {
    fn from(s: &str) -> Self {
        match s {
            "idle"         => Self::Idle,
            "recording"    => Self::Recording,
            "transcribing" => Self::Transcribing,
            "error"        => Self::Error,
            _              => Self::Inactive,
        }
    }
}

// ─── Genera un icono circular 16×16 con el color del estado ──────────────────

fn make_icon(argb: u32) -> Vec<ksni::Icon> {
    const S: i32 = 16;
    let cx = 7.5_f64;
    let cy = 7.5_f64;
    let r  = 7.0_f64;

    let a0 = ((argb >> 24) & 0xFF) as u8;
    let r0 = ((argb >> 16) & 0xFF) as u8;
    let g0 = ((argb >>  8) & 0xFF) as u8;
    let b0 = ( argb        & 0xFF) as u8;

    // Borde más oscuro (70 %)
    let (rb, gb, bb) = (
        (r0 as f64 * 0.7) as u8,
        (g0 as f64 * 0.7) as u8,
        (b0 as f64 * 0.7) as u8,
    );

    let mut data = Vec::with_capacity((S * S * 4) as usize);

    for y in 0..S {
        for x in 0..S {
            // sub-pixel AA (4 muestras por pixel)
            let mut hits = 0u32;
            for sy in 0..2 {
                for sx in 0..2 {
                    let px = x as f64 + sx as f64 * 0.5;
                    let py = y as f64 + sy as f64 * 0.5;
                    let dx = px - cx;
                    let dy = py - cy;
                    if dx * dx + dy * dy <= r * r {
                        hits += 1;
                    }
                }
            }

            if hits == 0 {
                data.extend_from_slice(&[0, 0, 0, 0]); // transparente
            } else {
                let frac = hits as f64 / 4.0;
                let dist = ((x as f64 - cx).powi(2) + (y as f64 - cy).powi(2)).sqrt();

                // último pixel ≈ borde
                let (cr, cg, cb) = if dist >= r - 1.0 {
                    (rb, gb, bb)
                } else {
                    (r0, g0, b0)
                };

                let aa = (a0 as f64 * frac) as u8;
                data.extend_from_slice(&[aa, cr, cg, cb]);
            }
        }
    }

    vec![ksni::Icon { width: S, height: S, data }]
}

// ─── JSON de voxtype ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct VoxtypeStatus {
    class: Option<String>,
    tooltip: Option<String>,
}

// ─── Tray ─────────────────────────────────────────────────────────────────────

struct VoxtypeTray {
    state:   Arc<Mutex<VoxtypeState>>,
    tip:     Arc<Mutex<String>>,
    quit_tx: Sender<()>,
}

impl ksni::Tray for VoxtypeTray {
    fn id(&self) -> String { "voxtype-tray".into() }

    fn icon_name(&self) -> String { String::new() }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        make_icon(self.state.lock().unwrap().color())
    }

    fn title(&self) -> String {
        "Voxtype".into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "Voxtype".into(),
            description: self.tip.lock().unwrap().clone(),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        let tx = self.quit_tx.clone();
        vec![
            StandardItem {
                label: "Salir".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(move |_| { let _ = tx.send(()); }),
                ..Default::default()
            }.into(),
        ]
    }
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    let state = Arc::new(Mutex::new(VoxtypeState::Inactive));
    let tip   = Arc::new(Mutex::new(VoxtypeState::Inactive.tooltip().to_string()));
    let (quit_tx, quit_rx) = mpsc::channel::<()>();

    let tray = VoxtypeTray {
        state: Arc::clone(&state),
        tip:   Arc::clone(&tip),
        quit_tx,
    };

    let handle = tray.spawn()
        .expect("No se pudo iniciar el system tray. ¿Está corriendo StatusNotifierWatcher?");

    let hw = handle.clone();
    let sw = Arc::clone(&state);
    let tw = Arc::clone(&tip);
    thread::spawn(move || watch_voxtype(hw, sw, tw));

    let _ = quit_rx.recv();
}

// ─── Watcher ──────────────────────────────────────────────────────────────────

fn watch_voxtype(
    handle: ksni::blocking::Handle<VoxtypeTray>,
    state:  Arc<Mutex<VoxtypeState>>,
    tip:    Arc<Mutex<String>>,
) {
    loop {
        let child = Command::new("voxtype")
            .args(["status", "--follow", "--format", "json"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();

        match child {
            Err(_) => {
                update(&handle, &state, &tip, VoxtypeState::Inactive, None);
                thread::sleep(std::time::Duration::from_secs(5));
            }
            Ok(mut child) => {
                let reader = BufReader::new(child.stdout.take().unwrap());
                for line in reader.lines() {
                    let Ok(line) = line else { break };
                    let line = line.trim().to_owned();
                    if line.is_empty() { continue; }
                    if let Ok(s) = serde_json::from_str::<VoxtypeStatus>(&line) {
                        let new_state = s.class.as_deref()
                            .map(VoxtypeState::from)
                            .unwrap_or(VoxtypeState::Inactive);
                        update(&handle, &state, &tip, new_state, s.tooltip);
                    }
                }
                let _ = child.wait();
                update(&handle, &state, &tip, VoxtypeState::Inactive, None);
                thread::sleep(std::time::Duration::from_secs(3));
            }
        }
    }
}

fn update(
    handle:    &ksni::blocking::Handle<VoxtypeTray>,
    state:     &Arc<Mutex<VoxtypeState>>,
    tip:       &Arc<Mutex<String>>,
    new_state: VoxtypeState,
    new_tip:   Option<String>,
) {
    let new_tip = new_tip.unwrap_or_else(|| new_state.tooltip().to_string());
    {
        let mut s = state.lock().unwrap();
        let mut t = tip.lock().unwrap();
        if *s == new_state && *t == new_tip { return; }
        *s = new_state;
        *t = new_tip;
    }
    handle.update(|_| {});
}
