# C56 — distribuição client0.13.0

- Prompt [C56 rev1–2](../clients/56-release-0.13.0.md) antes de bump e harness.
- Versão0.13.0 em Cargo, lock e Dockerfile. C52–C55 são a mudança funcional;
  phase5 global permanece aberta pelo broker/admin.
- Ubuntu26 após bump: 46 unitários, 33 wire, fmt, Clippy `-D warnings`, build
  release, inventário e 7 contratos de distribuição PASS.
- `scripts/verify_phase5_client.py` com broker0.14.0/client0.13.0 release em
  open-lab isolado: exclude/ACK/count, repeat/QoS1/interval, raw/binário e
  hex/binário PASS; processos/estado descartados. Nenhuma instância do
  operador foi lida ou alterada.
- Publicação: pendente neste checkpoint; preencher commit/tag, .deb, imagem,
  checksums, assets e confirmação remota após o ciclo de distribuição.
