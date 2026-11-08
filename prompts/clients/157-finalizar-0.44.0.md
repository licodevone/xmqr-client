# C157 v1.0 — fechar a distribuição Client 0.44.0

## Entrada e objetivo

Base `312ea86`, candidato C155, com `bench` C156 no diretório de trabalho.
Consolidar o benchmark na 0.44.0. Preservar a Bridge 0.5.0, o formato de
estado e as configurações do operador.

## Escopo e aceite

- Incluir C156, atualizar as notas e verificar `bench --help` no pacote
  e na imagem de CI. Acrescentar fixtures pequenas de reconexão e echo
  com broker TCP sintético, sem estado real.
- Rodar formatação, testes, Clippy, build e pacote. Confirmar os comandos
  e versões na instalação descartável, validar rollback, Windows Service
  e checksums. Medições de 1.000 clientes devem usar ambiente isolado.
- Enviar `release/validation-client-0.44.0`. Criar a tag anotada
  `v0.44.0` e publicar somente após aceite do commit e dos artefatos.
  Registrar as evidências em `prompts/registros/C157-finalizar-0.44.0.md`.
