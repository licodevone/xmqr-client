# C78 rev1 — propriedades de requisição e pares no publisher MQTT 5

Entrada: iniciar código somente após P141/C77 aceitos; client publicado 0.19.0. Este prompt precede a funcionalidade.

Objetivo: permitir `pub --protocol-version 5 --response-topic TOPICO`, `--correlation-data-hex HEX` (até 256 bytes, pares de dígitos hexadecimais) e `--publish-user-property CHAVE=VALOR` repetível (cada campo até 256 bytes, até 16 pares/2048 bytes codificados). Validar antes da rede, preservar ordem e duplicatas dos nomes, enviar pela API MQTT 5 junto com PFI, Content Type e Message Expiry anteriores. Rejeitar em MQTT 3.1.1 e `sub`. Não exibir esses metadados no subscriber nesta fatia nem tratar valores como segredos.

Aceite: parser/negativos, wire de propriedades combinadas e ordem, integração real QoS0/1/2 com broker P142 e Mosquitto, regressões C69/C77, fmt/test/Clippy/build/CI, docs/checklist honestos e commit/push. Sem tag/release para fatia isolada e sem ensaio físico ESP32.

Rev2 antes da correção: Clippy pediu `is_multiple_of(2)` e `as_chunks::<2>()` na leitura hexadecimal. Aplicar apenas essa equivalência e repetir gates.
