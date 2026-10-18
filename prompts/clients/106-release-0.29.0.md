# C106 — marco experimental client 0.29.0

Entrada: client independente 0.28.0 publicado; C104 (Session Expiry no DISCONNECT) e C105 (outbox explícita MQTT 5 QoS 1 Linux/WSL) aceitos após a tag anterior. Broker parceiro publicado 0.35.0, com P199/P200 em desenvolvimento; admin independente 0.10.0. Este prompt precede qualquer mudança de versão.

Objetivo: distribuir client independente 0.29.0 reunindo C104/C105. Atualizar Cargo/lock, Dockerfile, README, CHANGELOG, VERSIONING e nota de release com comandos de exemplo, semântica de DUP/PUBACK e limites. Outbox é opt-in, contém payload em claro sob permissões privadas e não restaura QoS 2 nem assinaturas. Fase 6B global e demais fases seguem abertas; não alterar broker/admin ou formato de estado do broker.

Aceite da fonte: fmt, suíte completa, Clippy `-D warnings`, build release Linux, inventário/licenças, contratos de distribuição, fixtures C104/C105 e integração com broker real (queda do client, broker restart, DUP com Session Present). CI do commit fonte exato, inclusive pacote Ubuntu 26.04 amd64 e units, antes de tag/release.

Aceite da distribuição: `.deb` 0.29.0-1, metadados e SHA-256; upgrade 0.28.0 → 0.29.0 → rollback em container Ubuntu 26.04 descartável, preservando configuração privada. Imagem Docker numerada 0.29.0 com usuário não privilegiado, versão/labels conferidas e execução por digest remoto. Tag anotada imutável `v0.29.0` e prerelease GitHub com manifesto, anexos, tamanhos e hashes remotos conferidos. Não mover Docker `latest`, instalar no WSL do operador nem publicar APT. Revisar prompt e registrar falhas antes de repetir gate essencial.

Evidência: registro C106 com hash da fonte/tag, CI, checksums, digest Docker, ciclo de pacote, URLs e limites. Marcar checklist apenas após publicação conferida. Admin independente permanece 0.10.0 e broker 0.35.0 até marco próprio.
