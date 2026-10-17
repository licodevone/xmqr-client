# C78 — propriedades de resposta e usuário no publisher MQTT 5

- [Prompt C78 rev1–rev2](../clients/78-publish-request-response-user-properties-mqtt5.md) criado antes do código. Base C77 aceita; fase 6C aberta.
- `pub --protocol-version 5` aceita Response Topic, Correlation Data hexadecimal e User Property repetível; rejeita modo/sub e argumentos inválidos antes da rede; preserva ordem e nomes duplicados. Assinante ainda não exibe esses metadados.
- Gates locais: 55 testes de CLI e 51 de wire PASS; integração real com broker P142 em QoS0/1/2, retained e restart PASS. Client C78 publicou com Response Topic, Correlation Data binária e pares ordenados num broker Mosquitto 2.0.22 descartável, com recebimento e inspeção do pacote por fixture independente PASS. Clippy `-D warnings`, fmt, build release e inventário de licenças PASS. Sem ensaio físico ESP32.
- Fonte `6820aad128ede9e9d85f4356ca297b1ce746ef9a` em `main`; [CI completa PASS](https://github.com/licodevone/xmqr-client/actions/runs/39000007226), incluindo pacote Ubuntu26 e unidades. Aceite C78 limitado às propriedades de publicação; assinante e fase 6C global permanecem abertos. Sem tag/release isolada.
