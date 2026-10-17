# C87 — Subscription Identifier MQTT 5 no client

- [Prompt C87](../clients/87-subscription-identifier-mqtt5.md) criado e commitado antes da funcionalidade. Fonte base: client `v0.22.0`/C86 aceito.
- CLI valida `--subscription-identifier` 1..268435455, duplicata, modo e versão antes da rede; CONNACK `Subscription Identifiers Available=0` é recusado antes do SUBSCRIBE. Um ID é enviado por SUBSCRIBE para todos os filtros. JSONL exibe lista ordenada com duplicatas; PUBLISH com ID zero é recusado.
- Testes de fio cobrem propriedade 0x0B, dois IDs no PUBLISH, recusa de CONNACK e ID zero. Integração Mosquitto 2.0.22 (`eclipse-mosquitto:2.0.22`) `C87_MOSQUITTO_PASS`: filtro simples recebe `[17]`; dois filtros sobrepostos resultam em duas cópias `[17]`, como permitido pelo padrão. Sem alegar que Mosquitto agregou em uma cópia `[17,17]`.
- Gates locais: 61 testes CLI e 62 de fio PASS; fmt, Clippy `-D warnings`, build release, inventário de licenças e 7 contratos de distribuição PASS. Commit exato e CI ainda pendentes. Broker XMQR ainda não implementa Subscription Identifier; fase 6D segue aberta.
