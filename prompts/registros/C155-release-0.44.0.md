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

## Candidato e publicação

- Commit `312ea86eef7b3ea9c7b3ebafa66a787282e87a43`.
- Push recusado por falha de resolução DNS de `github.com`. A tag e a
  prerelease aguardam o aceite.
- Em um terminal com acesso ao GitHub, acionar a CI do commit exato:

  ```powershell
  git push origin '312ea86eef7b3ea9c7b3ebafa66a787282e87a43:refs/heads/release/validation-client-0.44.0'
  ```

## Nova tentativa de publicação

Em 2026-11-08, o push do candidato exato falhou novamente na resolução
DNS de `github.com`. A consulta autenticada ao GitHub encontrou o SHA
ausente (HTTP 422) e a branch de validação e a tag ausentes (HTTP 404).
A tentativa alternativa de escrever um blob das notas retornou HTTP 403,
`Resource not accessible by integration`. A CI remota e os gates de
publicação permanecem bloqueados.
