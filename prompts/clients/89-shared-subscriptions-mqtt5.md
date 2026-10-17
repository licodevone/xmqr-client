# C89 rev1 — filtros compartilhados MQTT 5 no client

Entrada: client `v0.23.0` publicado, C87 Subscription Identifier aceito; broker XMQR 0.25.0 ainda não implementa shared subscriptions. [OASIS MQTT 5 §4.8.2](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html) define `$share/{ShareName}/{filter}`: nome não vazio sem `/`, `+` ou `#`, sem retained inicial, e uma entrega por grupo. Ausência da propriedade Shared Subscription Available no CONNACK significa suporte; valor 0 significa indisponibilidade.

Objetivo: aceitar explicitamente filtro compartilhado só em `sub --protocol-version 5`; validar estrutura antes da rede, recusar No Local em filtro compartilhado e recusar CONNACK que anuncie Shared Subscription Available=0. Preservar Subscription Identifier, QoS, JSONL, TLS, limites e filtros normais. Não adicionar retransmissão local nem alegar compartilhamento no broker XMQR atual.

Aceite: CLI negativos para versão/modo, grupo vazio/coringa, filtro vazio/inválido e No Local; wire para CONNACK 0 e SUBSCRIBE com filtro completo; integração Mosquitto 2.0.22 com dois consumidores de mesmo grupo, nenhuma duplicação por publicação, retained inicial ausente e coexistência com assinante normal; fmt/test/Clippy/build/licenças, CI do commit exato. Registrar parcela 6D sem marcar fase global ou criar release isolada.
