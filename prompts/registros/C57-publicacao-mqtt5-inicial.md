# C57 — publicação MQTT 5 inicial

- Prompt: [C57 rev2](../clients/57-publicacao-mqtt5-inicial.md), criado antes do código e refinado no recorte. Entrada: `main` em `f447488`, client 0.13.0.
- `--protocol-version 3.1.1|5` com default 3.1.1; 5 aceita apenas `pub` QoS0/1, Clean Start, sem Will, lote ou reconexão. Flag, perfil TOML e ambiente seguem precedência existente. Opções incompatíveis recusadas antes da rede. TLS/credenciais são reaproveitados; CONNACK recusado gera erro claro. PUBACK QoS1 é exigido antes do sucesso; publicação incerta não é reenviada.
- Ubuntu26/WSL: `cargo test --locked` PASS (47 unitários e 34 wire); `cargo clippy --locked --all-targets -- -D warnings` PASS; `cargo build --locked --bin mqtt-client` PASS; `cargo fmt --all -- --check` PASS; `python3 scripts/license_inventory.py --check` PASS; `git diff --check` no Git do Windows PASS.
- Fixture wire mede CONNECT5, CONNACK com limites, PUBLISH QoS0/1 com propriedade vazia, PUBACK, DISCONNECT e recusa. Integração com os binários reais do broker P115: `python3 ../xmqr/scripts/verify_mqtt5_cross.py` PASS para duas entregas a assinante3.1.1. Assinatura5, QoS2 e propriedades avançadas seguem pendentes.
- Mosquitto externo não instalado nesta máquina; interop externa MQTT5 permanece pendente. Sem tag/release isolada. CI GitHub e commit: pendentes de verificação.
