# C69 — opção Message Expiry MQTT 5

- [Prompt C69](../clients/69-message-expiry-mqtt5.md) criado antes do código. Base C68 `679f095`, release client0.16.0 intacta.
- `pub --protocol-version 5 --message-expiry-seconds 0..4294967295` envia a propriedade 0x02 em PUBLISH. CLI rejeita modo sub, 3.1.1, negativos, overflow e duplicatas antes de abrir rede.
- Validação local Ubuntu26/WSL: fmt, 49 unitários/43 wire, Clippy, build, vetor wire e integração real com broker P130/Mosquitto PASS. CI pendente.
- As demais propriedades de publicação e a retomada de fluxos QoS do próprio processo permanecem pendentes.
