# C160 — primeira rodada e correções

- FAIL na CI 45705105570. Duas fixtures MQTT fechavam o socket após a
  última resposta. Docker e DEB rejeitaram a falta do aviso revisado
  de windows-permissions 0.2.4.
- PASS local: cargo fmt --check e coleta locked (173 pacotes, 321 arquivos).
- BLOCKED local: cargo test --locked --test wire exige o linker MSVC ausente.
  A validação dos testes segue no GitHub Actions.

## Aceite de 25/11/2026

- PASS: commit 987844a40f8ae6b7167b3394ddea79301ec08a7e. Execução
  45706979355 aprovada nos seis jobs Ubuntu (Rust/Clippy, pacote, Docker
  e integrações). Execução 45706979145 aprovada no Windows, incluindo
  testes e build do adaptador de serviço.
- PASS: SHA-256 do DEB arquivado em dist/c160-ubuntu-987844a:
  08c1403e9cc1acf5e123ed87aeb150c4fc9cf47eabbceba1306d24fc4157fb81.
- PENDENTE: ensaio SCM com o Broker instalado na CI integrada P286.
