# C67 — expiração finita de sessão MQTT 5

- Prompt C67 rev1 criado antes do código; base C66 `37984ef`, CI C66 posteriormente PASS.
- `--session-expiry-seconds` aceita 0..4294967294 ou `never` somente em MQTT 5. O CONNECT transmite o valor exato. CLI rejeita negativos, overflow, duplicatas e valor numérico reservado `4294967295` (use `never`).
- Validação local Ubuntu26/WSL: fmt, 48 unitários/42 wire, Clippy, build e integração com broker P128 PASS. CI do commit pendente.
- A retomada de fluxos QoS pendentes por um novo processo do client e demais itens 6B permanecem abertos.
