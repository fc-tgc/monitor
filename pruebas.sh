#!/usr/bin/env bash
# Pruebas automaticas del monitor de heartbeats (TP2 - Parte 2.1).
# Corre la simulacion N veces (por defecto 20) y verifica en cada corrida:
#   1. Termina sola y en menos de 10 s (no queda colgada).
#   2. Los 3 nodos pasan de Desconocido a Vivo (3 lineas ALTA).
#   3. El nodo elegido pasa a Sospechoso exactamente una vez.
#   4. El nodo elegido pasa a Caido exactamente una vez.
#   5. Al final quedan exactamente 2 nodos Vivos.
# Uso: ./pruebas.sh [N]
set -u
N=${1:-20}
LIMITE_SEG=10
cargo build --release -q || exit 1
BIN=./target/release/monitor
SALIDA=$(mktemp)
trap 'rm -f "$SALIDA"' EXIT

# Ejecuta el binario con un limite de tiempo. Devuelve su codigo de salida
# (143 si hubo que matarlo por exceder el limite).
correr_con_limite() {
    "$BIN" > "$SALIDA" 2>&1 &
    local pid=$!
    ( sleep "$LIMITE_SEG"; kill "$pid" 2>/dev/null ) &
    local vigia=$!
    wait "$pid"
    local rc=$?
    kill "$vigia" 2>/dev/null
    wait "$vigia" 2>/dev/null
    return "$rc"
}

ok=0
for i in $(seq 1 "$N"); do
    correr_con_limite; rc=$?
    nodo=$(sed -nE 's/.*el Nodo ([0-9]+).*/\1/p' "$SALIDA")
    altas=$(grep -c "ALTA" "$SALIDA")
    sosp=$(grep -c "Nodo $nodo: Vivo -> Sospechoso" "$SALIDA")
    caido=$(grep -c "Nodo $nodo: Sospechoso -> Caido" "$SALIDA")
    vivos=$(grep -A3 "Estado final" "$SALIDA" | grep -c "Vivo")
    tiempos=$(grep "CAMBIO" "$SALIDA" | sed -nE 's/.*\(([0-9]+) ms sin latido\).*/\1/p' | tr '\n' ' ')
    if [ "$rc" = 0 ] && [ "$altas" = 3 ] && [ "$sosp" = 1 ] && [ "$caido" = 1 ] && [ "$vivos" = 2 ]; then
        ok=$((ok + 1)); estado="OK"
    else
        estado="FALLA (rc=$rc altas=$altas sosp=$sosp caido=$caido vivos=$vivos)"
    fi
    echo "Corrida $i: cae Nodo $nodo | ms sin latido al cambiar: $tiempos| $estado"
done
echo "Resultado: $ok/$N corridas correctas"
[ "$ok" = "$N" ]