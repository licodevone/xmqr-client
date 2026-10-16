# C64 — propriedades informativas de CONNECT MQTT 5 no client

- Prompt [C64 rev1–2](../clients/64-connect-properties-mqtt5.md) criado antes do código. Entrada `558edfa`, versão publicada 0.14.0 preservada.
- `pub|sub --protocol-version 5` aceita `--request-problem-info 0|1`, `--request-response-info 0|1` e `--connect-user-property CHAVE=VALOR` repetível. Pares preservam ordem/duplicidade, são limitados a 1024 bytes por campo e 4096 bytes codificados no conjunto. Os argumentos são rejeitados em MQTT 3.1.1; nenhum segredo é usado nas fixtures.
- Ubuntu26/WSL: testes CLI positivos/negativos e fixture CONNECT wire PASS. `cargo fmt --all -- --check`, `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` e `cargo build --locked --bins` PASS após rev2 extrair configuração de propriedades de `run_inner` para atender Clippy.
- Integração real com P123 no harness isolado PASS em pub/sub QoS1/2, ACL, UNSUBACK e Mosquitto 2.0.22. [CI `f68174b`](https://github.com/licodevone/xmqr-client/actions/runs/38983320426) PASS em Rust e pacote Ubuntu26; [broker corretivo `bb1084b`](https://github.com/licodevone/xmqr/actions/runs/38983737064) PASS.
- Sem perfil TOML/ambiente novos, versão, tag ou alteração da instalação WSL do operador. Fase 6A permanece aberta.
