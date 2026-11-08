# C155 v1.0 — preparar o candidato XMQR Client 0.44.0

## Entrada e escopo

Base local `f4394ff6787606ac686d95bf2ed7c17d2f3e5ed6`, com C154 e o
preflight C153 posteriores à tag publicada 0.43.0. Preparar a próxima
minor independente do Client, incluindo a base Windows C150–C151, sem
mudar a versão funcional `0.5.0` da bridge.

Alinhar `Cargo.toml`, `Cargo.lock`, os equivalentes do Windows Service,
`Dockerfile`, expectativas da fonte candidata em `.github/workflows/ci.yml`,
`VERSIONING.md` e notas em `docs/releases/0.44.0.md`. Preservar os pins dos
brokers e pacotes históricos usados como controles. Sem migração de estado.

## Validação e publicação

Validar versão dos dois binários, lockfiles, CI Ubuntu da fonte exata,
fixtures C154, pacote `.deb`, imagem numerada, upgrade/rollback e checksums.
Executar o gate do Windows Service para a fonte candidata. Registre o
resultado real em `prompts/registros/C155-release-0.44.0.md`.
Enviar primeiro uma branch de validação. Criar tag anotada `v0.44.0` e
prerelease somente após aceite integral. Preservar tags publicadas.
