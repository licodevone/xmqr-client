# C121 — distribuição experimental client 0.35.0

Entrada: 0.34.0 publicado; C119/C120 aceitos em `main`, permitindo outbox MQTT 5 QoS 1 por WSS mTLS e verificando retomada controlada. Broker P216 aceito em `main`. Este prompt precede mudança de versão, tag e release.

Objetivo: distribuir client independente 0.35.0 com outbox WSS QoS 1, preservando journals TCP e políticas de falha fechada; QoS 2 e assinatura persistente por URL continuam recusados. Sem migração de esquema.

Escopo: manifests/lock, versão CLI, changelog/README/versionamento, pacote Ubuntu 26.04 amd64, imagem Docker numerada e nota de prerelease. Não alterar WSL instalado, tags publicadas ou `latest`.

Gate: CI da fonte candidata exata (Rust/pacote), suite/fmt/Clippy/licenças, fixture de PUBACK perdido/Session Present, integração broker WSS/TCP, `.deb` com SHA-256/metadata, ciclo descartável upgrade/rollback 0.34.0 ↔ 0.35.0, imagem por digest remoto, tag imutável, prerelease e hashes remotos conferidos. Registrar PASS/FAIL/BLOCKED. Fases globais continuam abertas.
