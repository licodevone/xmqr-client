# C87 rev1 — Subscription Identifier MQTT 5 no client

Entrada: C86 `v0.22.0` publicado, C85 No Local/RAP aceito. [OASIS MQTT 5 §3.8.2.1.2 e §3.3.4](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html) define identificador 1..268435455 no SUBSCRIBE e seu retorno no PUBLISH, inclusive repetidos para filtros sobrepostos.

Objetivo: acrescentar `sub --protocol-version 5 --subscription-identifier N` a um SUBSCRIBE por execução; validar faixa, duplicata, modo e versão antes da rede; enviar propriedade 0x0B pela API pinada do rumqttc e recusar CONNACK que anuncie indisponibilidade. Mostrar a lista de IDs recebidos em `--output jsonl`, mantendo ordem e duplicatas. Não mexer em publisher ou em reconexão/replay de QoS local.

Aceite: testes CLI, JSONL e wire de SUBSCRIBE/PUBLISH; interoperabilidade real com Mosquitto 2.0.22 incluindo ID e filtros sobrepostos quando suportados, além de negativo para anúncio de indisponibilidade; fmt/test/Clippy/build/licenças e CI do commit exato. Atualizar README, changelog, registro e checklist como parcela da fase 6D. Sem tag/release isolada antes de outro marco de distribuição; não declarar fase 6D completa.
