# C145 — builds locais preservados fora do Git

Prompt publicado antes da alteração em `1cf06c8`, base funcional C144 `25c2e8a`.
Em 19/10/2026, `git rm --cached` retirou 1.861 arquivos gerados sob
`vendor/rumqttc-0.25.1/target/` do índice. Todos os 1.861 arquivos permaneceram
no disco e seus SHA-256 antes/depois coincidiram: zero ausentes/divergentes.
`git ls-files` desse diretório passou a retornar zero; `.gitignore` protege
`**/target/`, e a exclusão Docker preexistente continua.

`git diff --check` PASS. Sem alteração de Rust, dependências, estado, binários,
versão ou artefatos publicados; sem apagar diretório ou reescrever histórico.
Não se repetiram testes funcionais por uma alteração exclusiva de higiene.
O registro C144 agora distingue o teste local histórico da CI posterior
`39194165930` PASS da fonte `25c2e8a`. A distribuição própria de C144 continua
pendente, e conformidade integral continua sem demonstração.
