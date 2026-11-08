# Registro C155 — candidato XMQR Client 0.44.0

Data local: 2026-11-08, America/Sao_Paulo.
[Prompt C155 v1.0](../clients/155-release-0.44.0.md).
Base funcional `f4394ff6787606ac686d95bf2ed7c17d2f3e5ed6`.

- Manifestos, lockfiles, Dockerfile e expectativas da CI alinhados a
  `0.44.0`. A bridge mantém `0.5.0`; os pins de integração históricos
  foram preservados.
- PASS local: metadados dos dois manifestos com `--locked --offline
  --no-deps`, formatação, versões `mqtt-client 0.44.0` e
  `mqtt-bridge 0.5.0 (xmqr-client 0.44.0)` em builds diretos. A bridge
  ainda emite um aviso Windows preexistente de `unused_mut`. O Client
  passou no Clippy direto com os lints do manifesto e quatro testes CLI.
- BLOCKED: gates Cargo completos e wire, conforme registro C154.
  O check Cargo do Windows Service parou pela mesma dependência ausente.
  O Docker local recusou acesso ao engine.
  Aceite Ubuntu, Windows Service, pacote, imagem e hashes pendentes.
