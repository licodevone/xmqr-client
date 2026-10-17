# C82 — prerelease experimental client 0.21.0

- [Prompt C82](../clients/82-release-0.21.0.md) criado antes do bump. C80/P144 aceitos; C81 `dc2364f67ee22300bac0429d1df563d45e538425` passou na [CI](https://github.com/licodevone/xmqr-client/actions/runs/39001984037).
- Preparação: Cargo/lock, Dockerfile, changelog, README e nota de release atualizados. Broker parceiro 0.23.0 conserva documento v10; client não mantém estado QoS local para rollback.
- Gate local pré-commit PASS: fmt, 56 testes de CLI e 55 wire, Clippy `-D warnings`, sete testes de distribuição e inventário de licenças. Build release após commit da fonte exata PASS.
- Fonte imutável `9698a599be650f33028976e96244035472bf1059`, tag anotada `v0.21.0` no mesmo commit e [CI PASS](https://github.com/licodevone/xmqr-client/actions/runs/39002290418), incluindo pacote Ubuntu 26.04 amd64. `.deb` 0.21.0-1 e imagem Linux/amd64 conferidos quanto a versão, usuário não privilegiado e revisão.
- Upgrade 0.22.0/0.20.0 → 0.23.0/0.21.0, downgrade, remoção e purge em Ubuntu 26.04 amd64 descartável: `PACKAGE_LIFECYCLE_PASS`, preservando estado/configuração/senha dos pacotes conforme esperado. Sem alteração na instalação WSL do operador.
- [Prerelease client](https://github.com/licodevone/xmqr-client/releases/tag/v0.21.0) com seis anexos; nomes, tamanhos e SHA-256 remotos conferidos contra os arquivos locais. `.deb` SHA-256 `1724734731cd4c5741413205f3627ed015c824122ea42d9558b1c61c6595b4b5`. Docker Hub numerado `lepinheiro100/xmqr-client:0.21.0` publicado com digest remoto `sha256:53a7c5a8b00ba06b531a479a1b3d2872b33f41583de8eb20b9bc2e8b4a683655`; `latest` intacta.
- [Nota de release](../../docs/releases/0.21.0.md) registra limites. Fases 6A–6D e 7–9 continuam abertas.
