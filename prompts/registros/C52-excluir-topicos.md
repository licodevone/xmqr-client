# C52 — exclusão de tópicos recebidos

- Prompt: [C52 rev1–3](../clients/52-excluir-topicos.md), escrito antes do código.
- Entrada: main `55e8e462840fc487676d41852f9d0d26edb46d9f`, limpo;
  client0.12.0 publicado. Alterados parser, matcher MQTT local, sessão e wire.
- `--exclude-topic` aceita até 256 filtros únicos válidos, com limite1024 UTF-8
  e regra `$`; aplica-se após a recepção, antes da saída/contagem. ACKs QoS1/2
  foram observados no wire mesmo para tópicos excluídos; `--count 1` esperou a
  publicação não excluída. Não altera SUBSCRIBE nem ACL.
- Primeiro `cargo test` Windows falhou na montagem do frame da fixture;
  C52 rev2 corrigiu. Ubuntu26: 41 unitários e 30 wire PASS; Clippy apontou
  102 linhas em `parse_mode`; C52 rev3 extraiu validação. Fmt, Clippy
  `-D warnings`, build release e inventário PASS depois da correção.
- Integração real C56 com broker0.14.0 isolado: exclusão/ACK/count PASS.
  Sem migração de wire/estado/perfil.
