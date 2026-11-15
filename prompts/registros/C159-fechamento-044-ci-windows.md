# C159 — fechamento Windows preparado

2026-11-14, America/Sao_Paulo. [Prompt C159](../clients/159-fechamento-044-ci-windows.md).
Base `46f96b5`, com registros locais C155/C157/C158 preservados.

- Workflow Windows preparado para fmt, testes normais CLI/wire, Clippy,
  build locked independente do serviço, seus testes e probe de capacidade.
- O workflow P284 do Broker compila o MSI do Client pareado e executa
  o helper SCM contra o Broker instalado, com configuração sintética privada.
- PASS local: formato YAML e sintaxe PowerShell dos blocos run, em conjunto
  com a validação P284. Nenhum resultado remoto foi inferido dessas checagens.
- BLOCKED: novo fetch de windows-permissions falhou por DNS em crates.io.
  A validação de rede, compilação e empacotamento permanece para a CI.
  A execução Windows/Ubuntu e a publicação ainda não ocorreram.

Commit de validação `4a682706647730d485d4ce791f650e951e074d5f`.
O push para `release/validation-client-0.44.0` falhou por DNS. A CI não
foi disparada por esta sessão.
