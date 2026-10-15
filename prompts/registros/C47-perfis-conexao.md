# C47 — perfis de conexão

- Prompt: [C47 rev1–7](../clients/47-perfis-conexao.md).
- Entrada: `6ae0189`, client 0.9.0; implementação parcial preservada e concluída.
- Plataforma: Ubuntu 26.04/WSL, Rust local, fixtures privadas temporárias.
- Entrega: `--profile PATH` opt-in com TOML v1 limitado a 64 KiB, schema fechado,
  campos de conexão mTLS e caminhos relativos à pasta do arquivo. Flags
  explícitas vencem o perfil; ambiente de certificados mantém fallback.
- Gates finais: fmt, `cargo test --locked` (65 testes), Clippy `-D warnings`,
  build release e inventário de licenças PASS. Lockfile recebeu somente 13
  pacotes necessários às dependências novas; inventário atualizado.
- Integração real: client release 0.10.0 usou perfis em dois endpoints do broker
  release 0.12.0; QoS0/1/2 cruzados, retained após restart e negativos de
  CA/usuário/senha PASS em `../xmqr/scripts/verify_multi_endpoint.py`.
- Revisão de segurança: sem senha inline/payload no perfil, nenhum modo lab,
  mensagens de erro sem conteúdo do TOML; arquivo de senha continua sujeito
  à política privada do client. Cobertura limitada, sem rotação de credenciais.
- Compatibilidade: sem perfil, CLI existente preservada. C05/C04 parciais;
  rotação e TLS sem certificado de cliente ainda dependem de marcos futuros.
- Publicação: candidato à tag de código `v0.10.0` após revisão do diff.
