# C79 — prerelease experimental client 0.20.0

- [Prompt C79](../clients/79-release-0.20.0.md) criado antes do bump. Base funcional C78 `6820aad128ede9e9d85f4356ca297b1ce746ef9a` com [CI PASS](https://github.com/licodevone/xmqr-client/actions/runs/39000007226) e P142 `de6517c249f59153764919c5b36d8f477c362411` com [CI PASS](https://github.com/licodevone/xmqr/actions/runs/39000015530).
- Cargo/lock, Dockerfile, changelog, README e nota de release preparados. Gates da fonte exata, tag, pacote, imagem e prerelease pendentes. Broker parceiro 0.22.0 grava documento v10; rollback a 0.21.0 exige backup privado v8.
- Gate local pré-commit: fmt, 55 testes CLI + 51 wire, Clippy `-D warnings`, 7 testes de distribuição e inventário de licenças PASS. Build release e CI da versão 0.20.0 ainda pendentes.
