# C72 rev1 — Receive Maximum MQTT 5 no client

Entrada: `main` após C71. A CLI não expõe Receive Maximum em CONNECT MQTT 5.

Objetivo: aceitar `--receive-maximum 1..65535` em `sub --protocol-version 5`, validar antes da rede e configurar o CONNECT via rumqttc. Sem a opção, omitir propriedade. MQTT 3.1.1 e pub rejeitam a flag. A opção controla o máximo de PUBLISH QoS1/2 simultâneos recebidos nesta conexão.

Aceite: parser, wire, integração com broker P134 e Mosquitto, fmt/test/Clippy/build/CI. Não declarar flow control completo até máximo de pacote, envio e outros requisitos 6C.
