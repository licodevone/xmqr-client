# C73 — Maximum Packet Size MQTT 5 no client

- [Prompt C73](../clients/73-maximum-packet-size-mqtt5.md) criado antes do código; base `d77915b`, client 0.17.0.
- `mqtt-client sub --protocol-version 5 --maximum-packet-size 5..65536` anuncia `0x27` no CONNECT e aplica o teto de recepção via rumqttc. A opção é omitida por padrão e recusada em MQTT 3.1.1 ou `pub`; zero, abaixo de cinco, acima de 65536 e duplicata são erros antes da rede.
- Ubuntu 26.04/WSL: teste de parser e wire PASS; fmt/test/Clippy/build PASS. Integração TCP real com broker P136 e Mosquitto 2.0.22 PASS pelo script do broker. Commit `f130005` publicado; [CI de `main` PASS](https://github.com/licodevone/xmqr-client/actions/runs/38995798483).
- C73 aceito no recorte descrito. A faixa é uma limitação da CLI local de 64 KiB, não uma alteração do intervalo normativo MQTT 5.
