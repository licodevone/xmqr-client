# C80 rev1 — metadados MQTT 5 na saída JSONL do assinante

Entrada: C77/C78 e broker P141/P142 aceitos, releases 0.20.0/0.22.0 publicados; a fase 6C permanece aberta. Este prompt precede qualquer alteração funcional.

Objetivo: em `sub --protocol-version 5 --output jsonl`, manter os campos atuais de publicação e acrescentar `metadata` somente quando houver propriedades PUBLISH reconhecidas: Payload Format Indicator, Content Type, Response Topic, Correlation Data em hexadecimal reversível, User Property como lista ordenada de pares e Message Expiry Interval quando recebido. Não expor Topic Alias como propriedade de negócio. Preservar saída text/hex/raw e MQTT 3.1.1. Respeitar filtros, `--count`, limites de pacote e ACK QoS1/2. Não interpretar metadados como credenciais nem registrar valores em stderr.

Aceite: testes de serialização JSON, bytes binários/Unicode/duplicatas, wire MQTT 5, TCP real com broker 0.22.0 e Mosquitto, retained/restart e QoS0/1/2, regressões C75/C77/C78, fmt/test/Clippy/build/CI. Checklist parcial até CI; sem release isolada, sem ensaio físico ESP32. A fase 6C global só pode ser aceita após auditoria de flow control.
