# C95 — Message Expiry do Will MQTT 5 aceito parcialmente

O [prompt C95](../clients/95-will-message-expiry-mqtt5.md) precedeu a funcionalidade `89970491eb49d3cf2645ae87fa03015f43c255a8`. [CI do commit: PASS](https://github.com/licodevone/xmqr-client/actions/runs/39019851047), inclusive pacote Ubuntu 26.04. Localmente, 69 testes de CLI e 65 de wire, fmt, Clippy, build release, inventário de licenças e 7 contratos de distribuição passaram.

`--will-message-expiry-seconds` codifica a propriedade MQTT 5 `0x02` no Will do CONNECT, aceita 0..4294967295 e valida protocolo, Will completo e duplicatas antes da conexão. A integração real `xmqr/scripts/verify_mqtt5_will_expiry.py` passou com broker P164 em atraso maior que o intervalo, reinício, fila offline, retained e zero; regressões C94/C70 passaram. A propriedade de PUBLISH continua distinta.

Não houve tag isolada: C95 integra a release 0.26.0. Fases 6A/6B globais e interop ampla permanecem abertas.
