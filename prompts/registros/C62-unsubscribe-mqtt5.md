# C62 — UNSUBACK antes do encerramento de assinatura MQTT 5

- Prompt [C62 rev1–4](../clients/62-unsubscribe-mqtt5.md) criado antes da funcionalidade. Entrada `84ddaff`, versão 0.14.0 preservada.
- `sub --protocol-version 5 --count N` envia um UNSUBSCRIBE por filtro e aguarda UNSUBACK `Success` antes de DISCONNECT. Motivo recusado, quantidade incorreta de razões e timeout geram erro; nenhum cancelamento incerto é anunciado como sucesso. MQTT 3.1.1 preservado.
- Ubuntu26/WSL: teste wire positivo e negativos `0x11`, `0x87` e timeout PASS. `cargo fmt --all -- --check`, `cargo test --locked` (47 unitários, 37 wire), `cargo clippy --locked --all-targets -- -D warnings` e `cargo build --locked --bins` PASS. Primeiro Clippy FAIL por padrão OR não aninhado; corrigido na rev2.
- Integração real `scripts/verify_mqtt5_acl_reason.py` PASS com broker P121. O harness precisou corrigir captura do status e decodificar `payload_bytes` de JSONL (rev3–4); o fluxo MQTT encerrou com sucesso.
- [CI do commit funcional `3e17022`](https://github.com/licodevone/xmqr-client/actions/runs/38972742067) PASS, inclusive Rust e pacote Ubuntu26. Sem tag/release isolada; instalação WSL do operador não foi alterada.
