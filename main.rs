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

fn monitor(rx: Receiver<Heartbeat>) {
    let inicio = Instant::now();
    let mut ultimo_latido = [inicio; NUM_NODOS];
    let mut estados = [Estado::Desconocido; NUM_NODOS];

    while inicio.elapsed() < DURACION_SIMULACION {
        // recv_timeout se despierta con CUALQUIER latido; por eso el timeout
        // de cada nodo se calcula aparte con su propio Instant.
        match rx.recv_timeout(TICK_MONITOR) {
            Ok(hb) => {
                ultimo_latido[hb.id] = Instant::now();
                // Desconocido -> Vivo: primer heartbeat (alta del nodo).
                // Sospechoso/Caido -> Vivo: el nodo estaba lento, no caido
                if estados[hb.id] != Estado::Vivo {
                    println!(
                        "[Monitor] {:>5} ms  ALTA Nodo {}: {:?} -> Vivo",
                        inicio.elapsed().as_millis(),
                        hb.id,
                        estados[hb.id]
                    );
                    estados[hb.id] = Estado::Vivo;
                }
                println!(
                    "[Monitor] {:>5} ms  HEARTBEAT Nodo {} (seq {})",
                    inicio.elapsed().as_millis(),
                    hb.id,
                    hb.seq
                );
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                println!("[Monitor] Todos los nodos desconectados");
                break;
            }
        }

        // Revision de cada nodo despues de cada evento o timeout.
        for id in 0..NUM_NODOS {
            let silencio = ultimo_latido[id].elapsed();
            let nuevo = if silencio > UMBRAL_CAIDO {
                Estado::Caido
            } else if silencio > UMBRAL_SOSPECHOSO {
                Estado::Sospechoso
            } else {
                estados[id]
            };

            if nuevo != estados[id] {
                println!(
                    "[Monitor] {:>5} ms  CAMBIO Nodo {id}: {:?} -> {:?} ({} ms sin latido)",
                    inicio.elapsed().as_millis(),
                    estados[id],
                    nuevo,
                    silencio.as_millis()
                );
                estados[id] = nuevo;
            }
        }
    }

    println!("\n[Monitor] Estado final:");
    for (id, e) in estados.iter().enumerate() {
        println!("  Nodo {id}: {e:?}");
    }
}