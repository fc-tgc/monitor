use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const NUM_NODOS: usize = 3;
const INTERVALO_HEARTBEAT: Duration = Duration::from_millis(500);
const UMBRAL_SOSPECHOSO: Duration = Duration::from_secs(1);
const UMBRAL_CAIDO: Duration = Duration::from_secs(2);
const TICK_MONITOR: Duration = Duration::from_millis(100);
const DURACION_SIMULACION: Duration = Duration::from_secs(6);

#[derive(Debug)]
struct Heartbeat {
    id: usize,
    seq: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Estado {
    /// Agrego estado desconocido para definir el estado en el cual aun no recibi ningun heartbeat.
    Desconocido,
    Vivo,
    Sospechoso,
    Caido,
}

/// Evito usar rand y que tenga una dependencia externa.
fn semilla() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("reloj antes de 1970")
        .subsec_nanos()
}