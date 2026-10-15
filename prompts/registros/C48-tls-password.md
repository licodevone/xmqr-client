# C48 — client TLS com senha sem certificado

- Entrada: `4444c8086796c86974f644c177c4ad0ff6598961`, tag `v0.10.0`,
  árvore limpa; prompt [C48](../clients/48-tls-password.md) antes do código.
- `--tls-password` e `tls_password = true` no perfil TOML v1 exigem CA,
  usuário/senha, validam SAN/validade e dispensam identidade X.509 do client.
  Cert/key explícitos ou variáveis implícitas são recusados; mTLS segue padrão.
- Ubuntu 26.04/WSL: `cargo test --locked` PASS (38 testes de unidade e 29
  de integração); `cargo fmt --all -- --check`, `cargo clippy --locked
  --all-targets -- -D warnings` e `cargo build --locked` PASS antes do bump.
- Integração com broker P97 `scripts/verify_tls_password.py`: QoS0/1/2 e
  negativas senha/usuário/CA/SAN/ACL PASS. Regressão mTLS P96 PASS.
- Limites: rotação B11/E02 e políticas por listener B09 ainda pendentes.
