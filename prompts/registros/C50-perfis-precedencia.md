# C50 — perfis e precedência C05

- Prompt [C50](../clients/50-perfis-precedencia.md) antes do código; base
  `v0.11.1`/`47266aa687bc40d6d73b43e892268424eca9406b`, árvore limpa.
- TOML v1 adiciona `clean_session`, `reconnect_attempts` e
  `total_timeout_seconds`. Com perfil, flags explícitas vencem variáveis
  permitidas, que vencem arquivo e defaults. Valores de ambiente inválidos
  não retornam ao valor do arquivo e erros não ecoam conteúdo. Paths do arquivo
  são relativos à pasta dele; paths do ambiente/CLI ao cwd.
- Senha não pode aparecer inline; `--password-file`/perfil/ambiente são lidos
  pelo mesmo gate Linux de arquivo regular, dono e modo 0600. Sem perfil, a
  CLI anterior conserva comportamento e fallback de certificados.
- Ubuntu26/WSL: 40 testes unitários + 29 integração Rust PASS, fmt/Clippy
  `-D warnings`/build/inventário PASS após correção de lint. Integração externa
  em `../xmqr/scripts/verify_scoped_endpoints.py` (mTLS) e
  `../xmqr/scripts/verify_tls_password.py` (TLS-senha) PASS com perfil
  deliberadamente errado, ambiente corrigindo CA/porta e flag corrigindo host.
- Limites: não muda wire, estado ou certificados já estabelecidos; E02 físico
  permanece aberto por Wi-Fi externo. Publicação 0.12.0 em C51.
