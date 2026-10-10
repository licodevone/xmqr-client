# Registro C29 — assinatura de múltiplos tópicos

Data: 2026-10-10
Base: cliente independente com tag publicada `v0.3.0`, HEAD de entrada `d26eb9c`.
Candidato local: `0.4.0`; nenhuma tag, push ou release foi criada.

## Resultado

`sub` aceita de 1 a 256 filtros por repetição de `--topic` e envia um único
SUBSCRIBE; filtros duplicados e pacotes acima de 64 KiB são recusados antes da
conexão. `pub` mantém tópico único. O SUBACK deve conter uma concessão por filtro
e cada código é validado. Reconexão e `--count` continuam compartilhados pelo lote.

## Validação executada

Windows, no repositório `xmqr-client`:

- `cargo fmt --all -- --check`: PASS.
- `cargo test --locked --offline`: PASS — 41 testes (26 unitários, 15 wire).
- `cargo clippy --locked --offline --all-targets -- -D warnings`: PASS.
- `cargo build --locked --offline`: PASS.
- `python -m py_compile validation/C29-integration.py`: PASS.

Ubuntu-26.04/WSL, em `/mnt/d/projects/my-project/broker-client-rust/xmqr-client`:

- `cargo fmt --all -- --check`: PASS.
- `cargo test --locked --offline`: PASS — 43 testes (28 unitários, 15 wire).
- `cargo clippy --locked --offline --all-targets -- -D warnings`: PASS.
- `cargo build --locked --offline`: PASS.
- `python3 -m py_compile validation/C29-integration.py`: PASS.
- `python3 validation/C29-integration.py --broker ../xmqr/target/debug/mqtt-broker --client target/debug/mqtt-client`: PASS — um broker XMQR isolado validou filtros disjuntos/sobrepostos, entrega sem duplicata, retained e filtro explícito `$`.

## Limites

A integração usa um broker XMQR local em `open-lab`, com state temporário; ela
não demonstra interoperabilidade com Mosquitto nem mTLS com o broker. A etapa
não altera QoS por filtro: um `--qos` vale para todo o lote. Versão mínima exata
Rust 1.88 continua sem validação; os gates usam toolchains posteriores do Windows
e Ubuntu. A reconstrução independente 0.4.0 não foi ensaiada.
