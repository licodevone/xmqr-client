# C99 rev1 — criar sessão durável com ID atribuído no MQTT 5

Entrada: C97 pede Client Identifier vazio ao broker, mas a CLI só permite Session Expiry zero; C98 aceita um ID longo manual na reconexão. P166 já atribui IDs a sessões com expiração finita ou infinita. Este prompt precede a funcionalidade.

Objetivo: permitir `--server-assigned-client-id` com MQTT 5, Clean Start verdadeiro e `--session-expiry-seconds` entre zero e infinito, sem `--client-id` manual. Preservar a exigência de propriedade `0x12` válida no CONNACK e exibir o ID em stderr. Explicar que o operador deve guardar o ID para depois usar `--client-id ID --clean-start false` antes do vencimento; não prometer retomada automática do estado QoS local.

Aceite: teste da validação de combinações MQTT 5/3.1.1, Clean Start, ID manual, expiração 0/60/never; bytes CONNECT com ID vazio e propriedade de expiração; integração real CLI → broker P166 → CONNACK com ID → desconexão → Session Present=1 → reconexão CLI pelo ID, pub/sub e negativos. Executar suites Rust, fmt, Clippy, distribuição, CI exata, README/changelog e registro. Sem tag/release isolada automática; fase 6A/6B globais abertas.
