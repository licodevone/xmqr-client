# C49 — default de distribuição do client

- Entrada: tag de código `v0.11.0` em `59bcc34`, sem GitHub Release.
- Falha real: `python3 scripts/test_distribution.py` — 1/7 falhou porque
  `ARG VERSION=0.10.0` divergia de Cargo0.11.0. Corrigido após prompt
  [C49](../clients/49-default-versao-distribuicao.md).
- Solução: Dockerfile default e Cargo/lock0.11.1; runtime C48 preservado.
  A tag0.11.0 não foi reescrita e não será distribuída. O marco de publicação
  é 0.11.1 após gates/artefatos do novo commit.
- Após correção, `scripts/test_distribution.py`: 7/7 PASS. No Ubuntu26/WSL,
  fmt, 38 testes unitários + 29 de integração, Clippy `-D warnings`, build
  release e inventário MIT PASS. Artefatos serão avaliados antes da publicação.
