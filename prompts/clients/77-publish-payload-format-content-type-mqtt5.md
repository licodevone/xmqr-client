# C77 rev1 — flags de formato e tipo de conteúdo no PUBLISH MQTT 5

Entrada: client 0.19.0, broker P141 em implementação. Criar prompt antes do código.

Objetivo: permitir em `mqtt-client pub --protocol-version 5` as opções `--payload-format 0|1` e `--content-type TEXTO` (texto MQTT válido, até 256 bytes); ausentes, omitir ambas as propriedades. Enviar propriedades pela API MQTT 5 da rumqttc junto com Message Expiry já existente. Quando `--payload-format 1`, recusar payload que não seja UTF-8 bem formado antes da conexão; não impor interpretação de MIME ao Content Type. Rejeitar opções em `sub` e MQTT 3.1.1. Não alterar saída do assinante nesta fatia; metadata recebida pode ser verificada por fixture raw.

Aceite: parser/negativos e teste wire dos bytes das propriedades, publicação real QoS0/1/2 para broker P141 e Mosquitto quando compatível, regressão de Message Expiry, fmt/test/Clippy/build/CI, documentação honesta e commit/push. Sem tag/release para este recorte e sem ensaio físico ESP32.
