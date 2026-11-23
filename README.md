# Monitor de heartbeats — TP2 Programacion Concurrente (FIUBA)

Simulador de deteccion de fallos de la **Parte 2.1** del TP2.

Tres nodos coordinadores envian un *heartbeat* cada 500 ms a un monitor por un canal
`std::sync::mpsc`. El monitor usa `recv_timeout` y lleva el instante del ultimo latido de cada nodo:

| Tiempo sin latido | Estado |
|---|---|
| todavia no llego ninguno | `Desconocido` |
| ≤ 1 s | `Vivo` |
| > 1 s | `Sospechoso` |
| > 2 s | `Caido` |

Un nodo elegido al azar deja de enviar latidos despues de entre 2 y 5 envios (simula una falla de crash).

## Requisitos

- Rust, sin dependencias externas.

## Compilar y ejecutar

```bash
cargo build --release
cargo run --release
```

La simulacion dura 6 segundos y termina sola.

Para ver solo los cambios de estado, sin las lineas de cada latido:

```bash
cargo run --release | grep -v HEARTBEAT
```

## Pruebas

```bash
./pruebas.sh        # 20 corridas por defecto
./pruebas.sh 50     # o la cantidad que se indique
```

En cada corrida, el script verifica que el programa termine solo en menos de 10 s, que los 3 nodos
pasen de `Desconocido` a `Vivo`, que el nodo elegido pase una vez a `Sospechoso` y una vez a `Caido`, y
que al final queden 2 nodos `Vivo`. Funciona en macOS y en Linux, sin herramientas adicionales.
No hace falta tener el proyecto precompilado para correr las pruebas.
Probado en Linux y macOS, **no en Windows**.

## Salida esperada

```
Simulacion: el Nodo 0 caera tras 5 latidos

[Monitor]     0 ms  ALTA Nodo 2: Desconocido -> Vivo
[Monitor]     0 ms  ALTA Nodo 0: Desconocido -> Vivo
[Monitor]     0 ms  ALTA Nodo 1: Desconocido -> Vivo
[Nodo 0] CRASH: se cae (deja de enviar heartbeats)
[Monitor]  3000 ms  CAMBIO Nodo 0: Vivo -> Sospechoso (1000 ms sin latido)
[Monitor]  4001 ms  CAMBIO Nodo 0: Sospechoso -> Caido (2000 ms sin latido)

[Monitor] Estado final:
  Nodo 0: Caido
  Nodo 1: Vivo
  Nodo 2: Vivo
```

El nodo que cae y el momento de la caida cambian en cada ejecucion. El orden de las altas tambien puede cambiar, porque depende de que hilo envia primero.