# C96 rev1 — marco experimental client 0.26.0

Entrada: C94/C95 enviam metadados e Message Expiry do Will MQTT 5; broker P163/P164 é o par. Este prompt precede qualquer mudança de versão.

Objetivo: distribuir C94/C95 em client 0.26.0, atualizando Cargo/lock, Dockerfile, CHANGELOG, README, VERSIONING e nota de release. Documentar sintaxe Will separada das opções PUBLISH, limites e semântica do prazo iniciado quando o broker publica o Will. Fases 6A/6B globais permanecem abertas.

Aceite: bytes CONNECT, negativos CLI, TCP real P164 com atraso/restart/retained/offline, regressão C70/3.1.1, Rust/fmt/Clippy/build release/licenças/distribuição/CI exata; pacote Ubuntu 26.04 amd64 e upgrade/rollback descartável; imagem numerada executada por digest; tag anotada e prerelease com manifesto e SHA-256/tamanhos remotos conferidos. Não mover `latest` nem instalar no WSL do operador.
