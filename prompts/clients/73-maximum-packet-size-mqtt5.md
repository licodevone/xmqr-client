# C73 rev1 — Maximum Packet Size MQTT 5 no client

Entrada: `main` em `3353ee0` após C72; client 0.17.0. Fase 6C aberta.

Objetivo: expor `--maximum-packet-size 5..65536` em `sub --protocol-version 5`, validado antes da rede e enviado como propriedade CONNECT `0x27` via rumqttc. A faixa representa o teto local de 64 KiB e admite CONNACK mínimo de cinco bytes. Ausência omite a propriedade; MQTT 3.1.1 e `pub` recusam a flag. Explicar que um teto pequeno pode impedir SUBACK ou PUBLISH e encerrar a conexão.

Arquivos: CLI, sessão MQTT 5, README, testes de parser/wire, registro. Aceite: bytes exatos no CONNECT, 0/4/65537/duplicata, modo errado, integração com P136/Mosquitto e gates fmt/test/Clippy/build/CI. Não declarar flow control ou fase 6C completos. Fonte: [OASIS MQTT 5 §3.1.2.11.4](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html).
