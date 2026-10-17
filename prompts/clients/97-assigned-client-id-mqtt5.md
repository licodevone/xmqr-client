# C97 rev1 — usar Client Identifier atribuído no MQTT 5

Entrada: P166 aceitará CONNECT MQTT 5 com Client Identifier vazio e responderá com propriedade CONNACK `0x12`. A CLI hoje recusa ID vazio e sempre gera um padrão. Este prompt precede o código.

Objetivo: oferecer opção explícita para pedir ID atribuído pelo broker em uma sessão MQTT 5 limpa. Enviar Client Identifier vazio, exigir exatamente um identificador não vazio na resposta de sucesso e mostrá-lo em diagnóstico de stderr sem contaminar payload em stdout. Restringir a opção às sessões limpas/sem expiração persistente do client atual, pois a CLI não conserva estado de QoS para retomada automática. Reconexão automática da CLI deve tratar novo ID como nova sessão.

Aceite: parsing e negativos de modo/protocolo/flags, bytes CONNECT vazios, CONNACK com/sem propriedade, integração com P166 para pub/sub, regressão 3.1.1, Rust/fmt/Clippy/CI exata e registro factual. Não declarar fase 6A global concluída nem publicar release isolada automaticamente.
