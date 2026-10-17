# C100 rev1 — marco experimental client 0.27.0

Entrada: C97 solicita ID atribuído, C98 reutiliza ID longo e C99 cria sessão durável; P166 é o broker parceiro. Este prompt precede qualquer mudança de versão.

Objetivo: distribuir C97–C99 no client 0.27.0. Atualizar Cargo/lock, Dockerfile, CHANGELOG, README, VERSIONING e nota de release. Explicar o fluxo `--server-assigned-client-id --session-expiry-seconds 60` seguido de `--client-id ID --clean-start false`; stderr transporta o ID e stdout permanece só para payload. A CLI não salva estado QoS local. Fases globais 6A–6D e 7–9 permanecem abertas.

Aceite: bytes CONNECT/CONNACK, negativos CLI, integração P166 TCP real com sessão presente e pub/sub, regressão MQTT 3.1.1, Rust/fmt/Clippy/build release/licenças/distribuição/CI exata; pacote Ubuntu 26.04 amd64, upgrade/rollback descartável, imagem numerada por digest; tag anotada e prerelease com manifesto e SHA-256/tamanhos remotos. Não mover `latest` nem instalar no WSL do operador.
