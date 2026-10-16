# C58 — execução da assinatura MQTT 5 inicial

- Prompt: [C58 rev1–5](../clients/58-assinatura-mqtt5-inicial.md), criado antes do código. Entrada `0f10478`, 0.13.0.
- Implementado: `sub --protocol-version 5` QoS0/1 com SUBACK validado, filtros de saída, retained e count; PUBACK de recebimento e DISCONNECT antes da saída. Propriedades avançadas, QoS2 e reconexão fora do escopo.
- Ubuntu26/WSL: fmt PASS, 47 unitários PASS, 35 wire PASS, Clippy `-D warnings` PASS e build PASS. Teste wire próprio cobre SUBACK sucesso/falha, PUBLISH5 QoS1, PUBACK e DISCONNECT.
- Integração com broker P116: `scripts/verify_mqtt5_cross.py` PASS para retained e publicação viva de cliente MQTT3.1.1 em assinante MQTT5, além da direção inversa.
- Rev2 corrigiu CONNECT com propriedade extra; rev3 aceita PUBLISH antes de SUBACK conforme OASIS §3.8.4; rev4–5 corrigiram apenas fixtures de wire. Sem migração de estado ou alteração do padrão 3.1.1.
- Aceite PASS: commit `9aa86ae` e [CI](https://github.com/licodevone/xmqr-client/actions/runs/38951630275) concluída com sucesso, incluindo pacote Ubuntu26. Fase6A integral permanece aberta.
