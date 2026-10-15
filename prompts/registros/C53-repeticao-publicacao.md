# C53 — publicação repetida com intervalo

- Prompt: [C53 rev1](../clients/53-repeticao-publicacao.md), antes do código.
- `--repeat 1..1000` e intervalo1..60000 ms são aceitos somente em `pub`
  com mensagem/arquivo único. Espera monotônica após o ACK QoS1/2 ou envio
  QoS0; a próxima mensagem nunca é enfileirada antes disso. Ctrl+C e prazo
  total envolvem a espera. Erro de conexão/resultado incerto termina sem replay.
- Fixture wire observou três PUBLISH em QoS0/1/2, intervalos e ACKs; parser
  recusou fontes/modos/bordas inválidos. Ubuntu26: 42 unitários e 31 wire,
  fmt, Clippy `-D warnings`, build release e inventário PASS.
- Sem estado novo nem garantia de entrega de negócio; arquivo é lido uma vez.
- Integração real C56 com broker0.14.0 isolado: três publicações QoS1 com
  intervalo e três recepções PASS.
