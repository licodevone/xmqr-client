# C51 — distribuição experimental do client 0.12.0

- Prompt [C51](../clients/51-release-0.12.0.md); Cargo, lock e default Docker
  em 0.12.0. C50/C05 completa a precedência de perfis com env e campos de
  sessão/prazo; sem mudança no wire ou nas dependências.
- Ubuntu26/WSL após bump: fmt, 40 testes unitários + 29 testes de integração
  Rust, Clippy `-D warnings`, build release, inventário e 7 contratos de
  distribuição PASS. Os binários release client0.12.0/broker0.14.0 passaram
  integração externa mTLS/TLS-senha, com perfil/ambiente/flag e senha privada.
- Publicação: pendente neste checkpoint; preencher commit/tag, .deb, imagem,
  checksums, assets e confirmação remota após o ciclo de distribuição.
- E02 físico na COM4 permanece bloqueado pelo Wi-Fi privado indisponível;
  a fase4 global não é declarada completa por esta release.
