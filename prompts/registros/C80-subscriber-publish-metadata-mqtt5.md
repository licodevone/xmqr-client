# C80 — metadados MQTT 5 na saída JSONL do assinante

- [Prompt C80](../clients/80-subscriber-publish-metadata-mqtt5.md) criado antes do código. Base client 0.20.0 e broker 0.22.0 aceitos; fase 6C global aberta.
- Implementação: `sub` MQTT 5 conserva as propriedades PUBLISH na saída JSONL opcional; Correlation Data é hexadecimal e User Property mantém ordem/duplicatas. Topic Alias não é exposto como metadado. `sub` MQTT 3.1.1 e formatos text/hex/raw mantêm o contrato anterior.
- Gates locais PASS: 56 testes de CLI e 52 de wire, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, build release e inventário de licenças; TCP real QoS0/1/2, retained/restart, publisher XMQR e `mosquitto_pub` em [C80-subscribe-metadata.py](../../validation/C80-subscribe-metadata.py). Regressões P139/C75, P141/C77 e P142/C78 com scripts TCP reais PASS. Commit e CI pendentes; sem ensaio físico ESP32.
- Aceite C80 pendente da CI. Não marca a fase 6C global.
