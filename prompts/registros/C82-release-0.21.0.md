# C82 — prerelease experimental client 0.21.0

- [Prompt C82](../clients/82-release-0.21.0.md) criado antes do bump. C80/P144 aceitos; C81 `dc2364f67ee22300bac0429d1df563d45e538425` passou na [CI](https://github.com/licodevone/xmqr-client/actions/runs/39001984037).
- Preparação: Cargo/lock, Dockerfile, changelog, README e nota de release atualizados. Broker parceiro 0.23.0 conserva documento v10; client não mantém estado QoS local para rollback.
- Gate local pré-commit PASS: fmt, 56 testes de CLI e 55 wire, Clippy `-D warnings`, sete testes de distribuição e inventário de licenças. Build release, fonte exata, CI, pacote, tags e publicação pendentes.
