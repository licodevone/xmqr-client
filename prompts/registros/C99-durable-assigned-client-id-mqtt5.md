# C99 — sessão durável com ID atribuído aceita parcialmente

O [prompt C99](../clients/99-durable-assigned-client-id-mqtt5.md) foi commitado antes da funcionalidade (`bd3ecb8`). Fonte exata `a734c687e40f98843896c5f7aa1f7c2f5da704a4`: 76 testes CLI e 65 de wire PASS, fmt, Clippy, build release, inventário de licenças e 7 contratos de distribuição PASS localmente; [CI PASS](https://github.com/licodevone/xmqr-client/actions/runs/39022438646), incluindo pacote Ubuntu 26.04. O script parceiro no broker, fonte `02c57ae0a72617330e9e3b01c3be038ac4799ed3`, também teve [CI PASS](https://github.com/licodevone/xmqr/actions/runs/39022438104).

`--server-assigned-client-id` aceita Session Expiry 0, finita ou infinita em MQTT 5 com Clean Start verdadeiro. Mantém a recusa de ID manual simultâneo, Clean Start falso e MQTT 3.1.1; exige propriedade `0x12` válida no CONNACK e mostra o ID em stderr. O ensaio `P166_C97_C98_C99_ASSIGNED_ID_PASS` criou a sessão com a CLI, confirmou `Session Present=1` em reconexão, reutilizou o ID de 37 bytes na CLI e validou pub/sub. A CLI não persiste o próprio estado QoS local; o operador deve guardar o ID e reconectar antes da expiração.

Sem tag/release isolada. Aceites globais 6A/6B permanecem abertos.
