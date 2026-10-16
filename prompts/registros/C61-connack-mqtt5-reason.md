# C61 — motivo de CONNACK MQTT 5

- Prompt [C61 rev1](../clients/61-connack-mqtt5-reason.md) criado antes da funcionalidade; entrada `ec6af88`, versão preservada em 0.14.0.
- Client: recusa CONNACK mostra `ConnectReturnCode` recebido e byte hexadecimal sem expor senha, tópico ou payload. Falha de transporte segue separada; MQTT 3.1.1 não mudou.
- Ubuntu26/WSL: `cargo fmt --all -- --check`, `cargo test --locked` (47 unitários, 36 wire), `cargo clippy --locked --all-targets -- -D warnings` e `cargo build --locked --bins` PASS. Fixture CONNACK `0x83` PASS.
- Integração real `scripts/verify_mqtt5_acl_reason.py` PASS com broker P120: CONNACK `BadUserNamePassword (0x86)`, SUBACK/PUBACK ACL `NotAuthorized (0x87)`, publicação permitida.
- [CI remoto do commit funcional `1740368`](https://github.com/licodevone/xmqr-client/actions/runs/38971034003) PASS, inclusive Rust e pacote Ubuntu26. Sem tag/release isolada e sem alteração na instalação WSL do operador.
