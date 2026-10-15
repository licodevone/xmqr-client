# C55 — Will binário

- Prompt: [C55 rev1](../clients/55-will-binario.md), antes do código.
- `--will-message-file` lê arquivo regular com teto4096 antes de abrir rede;
  preserva NUL/UTF-8 inválido e recusa fontes simultâneas/sem tópico. Erros
  são fixos, sem path ou conteúdo. `--will-message` texto segue compatível.
- Fixture wire leu CONNECT real e comparou os bytes da mensagem Will. Testes
  de arquivo 4096/4097, ausente e diretório passaram. Ubuntu26: 46 unitários
  e 33 wire, fmt, Clippy `-D warnings`, build release e inventário PASS.
- Sem MQTT5, mudança de broker ou alegação de entrega de negócio.
