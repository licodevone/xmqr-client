# C64 rev1 — opções de propriedades informativas no CONNECT MQTT 5

Entrada: `main` `558edfa`, versão publicada 0.14.0, árvore limpa; C63 aceito. O client envia CONNECT 5 sem propriedades.

Objetivo: permitir `--request-problem-info 0|1`, `--request-response-info 0|1` e `--connect-user-property CHAVE=VALOR` repetível para `pub|sub --protocol-version 5`. Preservar ordem e duplicidade dos pares, validar campos UTF-8 e limites de pacote, rejeitar as novas opções em 3.1.1. O broker P123 aceita e não ecoa os pares; não há semântica de aplicação ou uso de credenciais nessas propriedades. Fonte: [OASIS MQTT 5 §§3.1.2.11.6–11.8](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html).

Escopo: CLI, `v5_session.rs`, fixtures wire e integração com P123; README/changelog/registro. Sem mudança de perfil TOML, ambiente, estado, versão, instalação ou modos TLS. Não permitir segredos em `--connect-user-property` nos exemplos.

Aceite: wire CONNECT 5 com flags e pares, negativos de duplicata de flag, valor inválido, par sem `=`, UTF-8 inválido e uso em 3.1.1; integração real pub/sub; fmt/test/Clippy/build/CI. MQTT 3.1.1 e QoS0/1/2 preservados. Apenas C64 parcial, 6A continua aberta.

Limite local escolhido antes do código: cada chave/valor UTF-8 até 1024 bytes, soma codificada das propriedades repetíveis até 4096 bytes; campos sem controles, NUL ou noncharacters. Isso mantém o CONNECT abaixo do limite de pacote existente e evita alocação excessiva por argumentos. Na CLI, argumento inválido significa texto Unicode proibido, pois `String` de Rust já é UTF-8 válido.

Rev2 após gate local: testes completos PASS, mas Clippy `too_many_lines` em `run_inner` (105/100) após configurar as novas propriedades. Extrair configuração para função pequena e repetir fmt/test/Clippy/build do client; não silenciar a regra.
