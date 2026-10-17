# C70 rev1 — Will MQTT 5 e Will Delay

Entrada: `main` após C69. O parser de Will já existe para MQTT 3.1.1, mas a trilha MQTT 5 recusa Will.

Objetivo: permitir Will no client MQTT 5 com `--will-topic`, `--will-message` ou arquivo, QoS/retain e `--will-delay-seconds 0..4294967295`; enviar a propriedade Will Delay (0x18) somente quando solicitada. Preservar 3.1.1, recusar delay sem Will ou com 3.1.1. Fonte: [OASIS MQTT 5 §3.1.3.2.2](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html).

Aceite: parser, vetor wire, integração anormal/normal com broker P132 e Mosquitto, fmt/test/Clippy/build e CI verde. Sem alegação de suporte às demais Will Properties.
