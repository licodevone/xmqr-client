# C52 rev1 — exclusão local de tópicos recebidos (C03)

2026-10-15. Entrada: `main` limpo em `55e8e462840fc487676d41852f9d0d26edb46d9f`,
client0.12.0 publicado. `sub` aceita filtros de assinatura, `--count` sobre
mensagens exibidas e filtros retained; não possui exclusão de tópicos.

Objetivo: `--exclude-topic FILTRO` repetível em `sub` suprime somente a saída
de PUBLISH cujo tópico concreto corresponde ao filtro. A assinatura enviada
ao broker, a responsabilidade dos ACKs QoS1/2, a sessão e os prazos não mudam.
`--count` conta apenas mensagens efetivamente exibidas após retained e exclusão.

Escopo: `src/cli.rs`, `src/session.rs`, testes de parser/wire pertinentes,
README, CHANGELOG, matriz e registro C52. Aceitar até 256 exclusões únicas,
com o mesmo limite de 1024 bytes UTF-8, validação MQTT e regra do namespace `$`
dos filtros de assinatura. Rejeitar flag em `pub`, duplicatas, wildcard inválido,
NUL e quota excedida antes de abrir rede. Não introduzir UNSUBSCRIBE ou ACL.

Compatibilidade: ausência da flag preserva comportamento e JSONL/text existentes;
sem mudança de wire, estado ou perfil. `--exclude-topic` é opção local explícita,
não campo TOML nem variável de ambiente. Não imprimir payloads descartados.

Aceite: fmt/test/Clippy/build; testes de `+`, `#`, níveis vazios, `$SYS`, UTF-8,
duplicata/quota, retained combinado, contagem e ACK QoS1/2 mesmo quando o
PUBLISH é filtrado. Integração com broker isolado e cliente independente quando
o harness existente permitir. Registrar resultado real; não marcar toda a fase 5
por esta fatia.

## rev2 — fixture wire

O primeiro `cargo test` recusou somente a montagem tipada do frame de teste:
um array fixo de cabeçalho e um `Vec<u8>` não formam automaticamente a mesma
slice. Construir o frame explicitamente com capacidade limitada, sem mudar o
comportamento do client, e repetir os gates.

## rev3 — limite de complexidade do parser

Os 41 testes unitários e 30 testes wire passaram no Ubuntu26, mas Clippy
recusou `parse_mode` com 102 linhas. Extrair a validação das exclusões para
função pequena, preservando as mesmas recusas, e repetir Clippy/build/inventário.
