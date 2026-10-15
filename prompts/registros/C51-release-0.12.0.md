# C51 — distribuição experimental do client 0.12.0

- Prompt [C51](../clients/51-release-0.12.0.md); Cargo, lock e default Docker
  em 0.12.0. C50/C05 completa a precedência de perfis com env e campos de
  sessão/prazo; sem mudança no wire ou nas dependências.
- Ubuntu26/WSL após bump: fmt, 40 testes unitários + 29 testes de integração
  Rust, Clippy `-D warnings`, build release, inventário e 7 contratos de
  distribuição PASS. Os binários release client0.12.0/broker0.14.0 passaram
  integração externa mTLS/TLS-senha, com perfil/ambiente/flag e senha privada.
- Publicação: tag anotada imutável `v0.12.0` no commit
  `2dbc4aff4713b4b7865446a5cac446f0ab7f4afe`, push de `main`/tag e
  [prerelease GitHub](https://github.com/licodevone/xmqr-client/releases/tag/v0.12.0)
  confirmados. Os 6 assets (.deb Ubuntu 26.04 amd64, export Docker, manifesto
  e seus SHA-256) tiveram digests remotos iguais aos arquivos locais;
  manifesto aponta para esse commit.
- Imagem numerada Docker Hub `lepinheiro100/xmqr-client:0.12.0` confirmada,
  sem mover `latest`, digest
  `sha256:51fdd7ab34964d8d9294e05fc44397f234a22168973dfb521b0e6097ffec7e30`.
- E02 físico na COM4 passou com seis rotações, conforme P102 no repositório do
  broker. O aceite da fase4 tem escopo finito e limites explícitos.
