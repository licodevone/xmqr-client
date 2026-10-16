# C63 — QoS 2 MQTT 5 no client

- Prompt [C63 rev1–7](../clients/63-qos2-mqtt5.md) criado antes da funcionalidade. Entrada `9c6e77c`, versão publicada 0.14.0 preservada.
- `pub --protocol-version 5 --qos 2` espera PUBCOMP antes de relatar sucesso; recusa PUBREC/PUBCOMP mostra motivo e código. `sub --protocol-version 5 --qos 2` aceita SUBACK QoS2 e conclui o ciclo inbound antes de DISCONNECT, inclusive quando PUBREL chega durante a espera por UNSUBACK. MQTT 3.1.1 preservado.
- Ubuntu26/WSL: fixtures wire positivas e negativas QoS2 PASS, incluindo PUBREC `0x87` e PUBCOMP `0x92` sem saída de sucesso. `cargo fmt --all -- --check`, `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` e `cargo build --locked --bins` PASS. A fixture inicial esperava payload textual em vez de bytes; rev2 corrigiu e repetiu.
- Integração real com broker P122 via `scripts/verify_mqtt5_acl_reason.py` PASS em QoS1/2, ACL e UNSUBACK. Interop Mosquitto 2.0.22 MQTT5 QoS2 PASS nas duas direções. O primeiro ensaio externo falhou porque Mosquitto usou Client ID vazio; IDs explícitos e uma espera curta de inicialização resolveram o harness, sem alteração do broker (rev4–7).
- [CI do commit funcional `2b1ba78`](https://github.com/licodevone/xmqr-client/actions/runs/38974179840) PASS, incluindo Rust e pacote Ubuntu26. Sem tag/release isolada; instalação WSL do operador não foi alterada.
