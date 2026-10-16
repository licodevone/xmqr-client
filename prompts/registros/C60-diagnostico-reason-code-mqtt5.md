# C60 — diagnóstico MQTT 5 de PUBACK/SUBACK

- Prompt: [C60 rev1–6](../clients/60-diagnostico-reason-code-mqtt5.md), criado antes do código. Entrada `5eab7d4`, 0.14.0.
- Implementado: erro de PUBACK MQTT 5 exibe enum e código hexadecimal; SUBACK recusado indica índice do filtro e reason code; downgrade de QoS é identificado como alteração, sem imprimir tópico, credenciais ou payload. Ajuda CLI agora descreve pub/sub MQTT 5. MQTT 3.1.1 e saída de sucesso permanecem.
- Ubuntu26/WSL: `cargo fmt --all -- --check` PASS; `cargo test --locked` PASS (47 unitários, 36 wire); `cargo clippy --locked --all-targets -- -D warnings` PASS; `cargo build --locked --bins` PASS. Teste wire inclui PUBACK `0x87`, SUBACK `0x80`/`0x87`, downgrade de QoS e sucessos.
- Integração `scripts/verify_mqtt5_acl_reason.py` PASS com broker P119 real: publicação negada mostra `NotAuthorized (0x87)` e falha sem saída de sucesso; publicação autorizada passa; assinatura negada mostra `SUBACK ... 0x80`.
- FAIL corrigidos e repetidos: fixture de PUBACK fechava socket cedo (rev2); ID de client da integração excedia 23 caracteres (rev3); dois lints novos de Clippy (rev4). Rev5 separou downgrade de recusa antes do aceite.
- Rev6: ajuda `mqtt-client --help` PASS com `MQTT5 inicial pub/sub QoS0/1`; testes, fmt, Clippy e build repetidos PASS após a correção textual. Inventário de licenças com Python Windows FAIL por CP1252 ao ler `LICENSE`; Ubuntu26/WSL PASS.
- Commit funcional `2200db1`; [CI](https://github.com/licodevone/xmqr-client/actions/runs/38966376915) PASS, incluindo Rust e pacote Ubuntu26. Sem bump/tag/release isolados; cliente instalado no WSL do operador não foi substituído.
