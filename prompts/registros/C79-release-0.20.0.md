# C79 — prerelease experimental client 0.20.0

- [Prompt C79](../clients/79-release-0.20.0.md) criado antes do bump. Base funcional C78 `6820aad128ede9e9d85f4356ca297b1ce746ef9a` com [CI PASS](https://github.com/licodevone/xmqr-client/actions/runs/39000007226) e P142 `de6517c249f59153764919c5b36d8f477c362411` com [CI PASS](https://github.com/licodevone/xmqr/actions/runs/39000015530).
- Na preparação, Cargo/lock, Dockerfile, changelog, README e nota de release foram atualizados. Broker parceiro 0.22.0 grava documento v10; rollback a 0.21.0 exige backup privado v8.
- Gate local pré-commit: fmt, 55 testes CLI + 51 wire, Clippy `-D warnings`, 7 testes de distribuição e inventário de licenças PASS. Build release executado após o commit da fonte exata.
- Fonte imutável `96843b46dd4bb75d598c21605d04dce9a64ec04d`, tag anotada `v0.20.0`, [CI completa PASS](https://github.com/licodevone/xmqr-client/actions/runs/39000329167), incluindo pacote Ubuntu26 e unidades. `.deb` 0.20.0-1 e imagem Linux/amd64 numerada gerados da fonte exata; versão, usuário 10001 e revisão conferidos.
- Upgrade conjunto 0.21.0/0.19.0 → 0.22.0/0.20.0, downgrade e remoção em contêiner Ubuntu 26.04 descartável: `PACKAGE_LIFECYCLE_PASS`, preservando estado/configuração; WSL do operador inalterado.
- [Prerelease](https://github.com/licodevone/xmqr-client/releases/tag/v0.20.0) com seis anexos: `.deb`, imagem, dois SHA-256, manifesto e SHA-256 do manifesto. Nomes, tamanhos e SHA-256 de todos os anexos remotos conferidos. Docker Hub numerado `lepinheiro100/xmqr-client:0.20.0`, digest remoto `sha256:03f0c94ac5d0f552b5900ef6de33919613bd5982b1e61033263364eca03e4010` conferido; `latest` intacta.
- [Nota de release](../../docs/releases/0.20.0.md) documenta rollback e limites. Fases 6A–6D e 7–9 seguem abertas.
