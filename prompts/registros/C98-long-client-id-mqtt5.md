# C98 — reutilização manual do ID MQTT 5 aceita parcialmente

O [prompt C98](../clients/98-long-client-id-mqtt5.md) foi commitado antes da funcionalidade (`0077a7a`). Fonte exata `dbfb07c91936d7659544000ac38db6d0ae43f485`: 75 testes de CLI e 65 de wire PASS, fmt, Clippy, build release, inventário de licenças e 7 contratos de distribuição PASS localmente; [CI PASS](https://github.com/licodevone/xmqr-client/actions/runs/39022188306), incluindo pacote Ubuntu 26.04.

`--client-id` aceita 1..254 caracteres ASCII alfanuméricos, `-` ou `_` em MQTT 5; MQTT 3.1.1 mantém 1..23. Os testes cobrem 0/23/24/37/254/255, os bytes CONNECT de um ID atribuído de 37 bytes e a retomada real de sessão durável no broker P166 (`P166_C97_C98_ASSIGNED_ID_PASS`). A CLI não restaura automaticamente o estado QoS local nem promete interoperabilidade de IDs longos com qualquer broker.

Sem tag/release isolada. Aceite global da fase 6A permanece aberto.
