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
- Publicação: commit/tag imutável `35dbfa9`/`v0.13.0` enviados ao GitHub;
  [prerelease](https://github.com/licodevone/xmqr-client/releases/tag/v0.13.0)
  contém seis assets (.deb Ubuntu26 amd64, Docker export, hashes e manifesto).
  Digests SHA-256 dos seis assets no GitHub conferem com os arquivos locais.
  Docker `lepinheiro100/xmqr-client:0.13.0` publicado com digest remoto
  `sha256:126e6f2e2ba00ea7b88c3015e57846b3e870101495299dfa456afbb03fbf0e2e`.
  `latest` não foi alterada. Rollback: pacote/imagem 0.12.0.
