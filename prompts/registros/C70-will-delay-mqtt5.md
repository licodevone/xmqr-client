# C70 — Will MQTT 5 e Will Delay

- [Prompt C70](../clients/70-will-delay-mqtt5.md) criado antes do código; base C69 `d60eba1` com [CI PASS](https://github.com/licodevone/xmqr-client/actions/runs/38991018412).
- `mqtt-client pub --protocol-version 5` aceita Will e `--will-delay-seconds 0..4294967295`, que exige Will. MQTT 3.1.1 rejeita a opção de delay; sem a opção, o wire omite a propriedade.
- Testes de parser e wire, integração anormal/normal com broker P132 e Mosquitto, fmt/test/Clippy/build PASS. Commit `f64ddd9`, [CI](https://github.com/licodevone/xmqr-client/actions/runs/38991792436) PASS.
- C70 cobre Will Delay; outras Will Properties MQTT 5 e retomada durável de QoS do próprio client seguem pendentes.
