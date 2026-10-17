# C69 rev1 — opção Message Expiry MQTT 5

Entrada: `main` após C68, client0.16.0 publicado. O client publica MQTT 5 sem propriedades PUBLISH.

Objetivo: `mqtt-client pub --protocol-version 5 --message-expiry-seconds N` envia a propriedade Message Expiry Interval 0x02 exata; apenas modo `pub`, valor decimal 0..4294967295. Manter comandos anteriores e impedir uso em 3.1.1/sub antes da rede. Fonte: [OASIS MQTT 5 §3.3.2.3.3](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html).

Aceite: parser, vetor wire e integração com broker P130/Mosquitto; fmt/test/Clippy/build e CI verde. Não incluir demais propriedades de publicação neste recorte.
