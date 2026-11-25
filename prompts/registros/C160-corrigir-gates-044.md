# C160 — primeira rodada e correções

- FAIL na CI 45705105570. Duas fixtures MQTT fechavam o socket após a
  última resposta. Docker e DEB rejeitaram a falta do aviso revisado
  de windows-permissions 0.2.4.
- PASS local: cargo fmt --check e coleta locked (173 pacotes, 321 arquivos).
- BLOCKED local: cargo test --locked --test wire exige o linker MSVC ausente.
  A validação dos testes segue no GitHub Actions.
