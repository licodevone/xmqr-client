# C66 — opções de sessão MQTT 5 zero/infinita

- Prompt C66 rev1 criado antes da implementação; base `main` pós-C64, versão publicada v0.15.0.
- CLI aceita `--clean-start true|false` e `--session-expiry-seconds 0|never` apenas em MQTT 5. Rejeita mistura com `--clean-session`, valores inválidos e duplicatas. O CONNECT transmite os campos e o client recusa CONNACK com Session Present 1 após Clean Start 1.
- Validação local Ubuntu26/WSL: `cargo fmt --all -- --check`, `cargo test --locked` (48 unitários e 42 wire), `cargo clippy --locked --all-targets -- -D warnings` e `cargo build --locked` PASS após o último ajuste. Integração broker P127 com publicação QoS1 offline e regressão cruzada PASS.
- [CI do commit `37984ef`](https://github.com/licodevone/xmqr-client/actions/runs/38989241925) PASS. Fluxos QoS em andamento não são restaurados por novo processo; intervalos finitos foram encaminhados a C67 e demais requisitos 6B permanecem abertos. Sem versão/tag/release nesta fatia.
