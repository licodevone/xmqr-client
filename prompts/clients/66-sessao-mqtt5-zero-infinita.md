# C66 rev1 — Clean Start e expiração 0/infinita MQTT 5

Entrada: `main` após C64 e tag client v0.15.0, árvore limpa; o client fixa Clean Start 1 e não expõe Session Expiry.

Objetivo: expor `--clean-start true|false` e `--session-expiry-seconds 0|never` em `pub|sub --protocol-version 5`, enviando as propriedades de CONNECT corretas e reconhecendo Session Present no CONNACK. Fonte: [OASIS MQTT 5 §§3.1.2.4, 3.1.2.11.2 e 3.2.2.1](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html).

Escopo: CLI, sessão MQTT5, README, testes unitários/wire e integração com P127. Preservar `--clean-session` no MQTT 3.1.1; rejeitar mistura das opções de protocolo. Expiração finita, Will e retomada de fluxos QoS em um novo processo permanecem fora desta fatia; não declarar 6B completo.

Aceite: opções corretas no CONNECT para cada combinação, validação de duplicata/valor/protocolo incompatível, retomada de assinatura/offline com broker e Mosquitto, regressão 3.1.1; fmt, test, Clippy e build.
