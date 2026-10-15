# C54 — formatos de saída binária

- Prompt: [C54 rev1](../clients/54-formatos-saida.md), antes do código.
- `sub --output hex` produz tópico/payload UTF-8 bytes em hex com QoS/retain;
  `sub --output raw --count 1` escreve exatamente os bytes de uma mensagem,
  sem separador. Status vai a stderr; falha de stdout propaga erro. Text/jsonl
  permanecem compatíveis. Não há timestamp de publicação sintético.
- Testes com NUL, UTF-8 inválido, Unicode, newline de hex e ausência de newline
  raw; parser recusou raw sem count1 e formatos binários em pub. Ubuntu26:
  44 unitários e 32 wire, fmt, Clippy `-D warnings`, build release e
  inventário PASS.
- Sem mudança de wire MQTT, ACL ou payload; raw é deliberadamente de uma
  única mensagem para não produzir stream sem framing.
- Integração real C56 com broker0.14.0 isolado: bytes binários raw/hex PASS.
