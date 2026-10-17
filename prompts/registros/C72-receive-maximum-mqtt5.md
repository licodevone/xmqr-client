# C72 — Receive Maximum MQTT 5 no client

- [Prompt C72](../clients/72-receive-maximum-mqtt5.md) criado antes da implementação. Base `fef96f0`, client 0.17.0; sem mudança de versão ou de formato persistido.
- `mqtt-client sub --protocol-version 5 --receive-maximum 1..65535` valida a opção antes da rede e codifica a propriedade CONNECT. Valor ausente omite a propriedade. Valor zero, acima de 65535, uso em MQTT 3.1.1 e uso em `pub` são recusados.
- Ubuntu 26.04/WSL: `cargo fmt --all --check`, `cargo test --locked --quiet`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo build --locked --release --quiet` PASS. O teste wire inspeciona o CONNECT. Interoperabilidade TCP real com broker P134 e Mosquitto 2.0.22 PASS pelo script `scripts/verify_mqtt5_receive_maximum.py` do broker.
- Aceite de C72 condicionado à CI de `main` após publicação. Não representa flow control MQTT 5 completo.
