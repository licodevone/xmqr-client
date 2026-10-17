# C67 rev1 — Session Expiry finita MQTT 5

Entrada: `main` após C66 (`37984ef`), v0.15.0 publicada, CI C66 em execução. CLI só aceita `--session-expiry-seconds 0|never`.

Objetivo: aceitar valores decimais 1..0xFFFFFFFE em `--session-expiry-seconds` para MQTT 5 e enviar o valor exato no CONNECT. Preservar `0` e `never`, escopo MQTT3, ClientID, TLS e validação de opções. Fonte: [OASIS MQTT 5 §3.1.2.11.2](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html).

Aceite: limites 0/1/máximo/never, inválidos/overflow/duplicatas, wire e integração com broker P128/Mosquitto; fmt/test/Clippy/build. Não prometer retomada de fluxos QoS por novo processo.
