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

/// Hilo de un nodo. Si `latidos_hasta_caer` es Some(n), envia n latidos y muere.
fn nodo(id: usize, tx: Sender<Heartbeat>, latidos_hasta_caer: Option<u64>) {
    let mut seq = 0;
    loop {
        if latidos_hasta_caer == Some(seq) {
            println!("[Nodo {id}] CRASH: se cae (deja de enviar heartbeats)");
            return;
        }

        if tx.send(Heartbeat { id, seq }).is_err() {
            return;
        }
        seq += 1;
        thread::sleep(INTERVALO_HEARTBEAT);
    }
}