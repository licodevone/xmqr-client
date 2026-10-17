# C98 rev1 — reutilizar identificador atribuído pelo broker

Entrada: C97 mostra em stderr o Client Identifier de 37 bytes atribuído por P166; a CLI ainda limita `--client-id` a 23 bytes, inclusive em MQTT 5. O limite do broker é 256 bytes e listeners com escopo interno reservam 2 bytes. Este prompt precede o código.

Objetivo: aceitar em `--client-id` MQTT 5 identificadores ASCII alfanuméricos, `-` ou `_` com 1..254 bytes, mantendo a validação anterior de 1..23 em MQTT 3.1.1. Assim o usuário pode copiar o ID atribuído e solicitar uma conexão posterior com esse mesmo ID. Não prometer retomada do estado local de QoS da CLI; preservar as regras já existentes de Clean Start e Session Expiry.

Aceite: limites 23/24/254/255 por protocolo, ID vazio recusado quando explícito, bytes CONNECT com ID de 37 bytes e reconexão real ao broker P166 para uma sessão durável QoS0, suites Rust/fmt/Clippy, integração e CI exata. Registrar que suporte a ID longo é compatibilidade com XMQR, não garantia para qualquer broker externo. Sem release isolada automática nem aceite global 6A.
