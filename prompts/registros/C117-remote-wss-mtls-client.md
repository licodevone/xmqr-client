# C117 — client WSS mTLS remoto com opt-in explícito

- [Prompt C117](../clients/117-remote-wss-mtls-client.md) precedeu o código (`4a1df87`). Implementação: `06d0c74`. [CI 39119944272](https://github.com/licodevone/xmqr-client/actions/runs/39119944272) da fonte funcional: Rust e pacote Ubuntu 26.04 PASS.
- `--allow-remote-wss` autoriza destino WSS fora de loopback, inclusive `ws_url` de perfil C116; sem a flag o destino é recusado. CA/SAN, certificado/chave cliente, senha MQTT, subprotocolo `mqtt` e TLS permanecem obrigatórios. URLs com userinfo, query, fragmento e destinos ambíguos são recusados; não há fallback para WS aberto.
- Suíte (93 unitários e 66 wire), fmt, Clippy, distribuição, licenças e fixture TLS PASS. Com broker P214 em Docker descartável, pub/sub MQTT 3.1.1/5 por WSS remoto e perfil, regressão TCP/restart, negativos de opt-in/CA/SAN/senha/ACL: `P214_C117_REMOTE_WSS_PASS`.
- Aceite parcial da fase 7. O código está em `main` após a tag 0.33.0 e não integra esse pacote. Navegador e estado durável por URL permanecem abertos.
