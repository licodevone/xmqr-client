# C59 — preparação da release 0.14.0

- Prompt [C59](../clients/59-release-0.14.0.md) criado antes do bump.
- Base funcional `9aa86ae`: [CI](https://github.com/licodevone/xmqr-client/actions/runs/38951630275) PASS, incluindo pacote Ubuntu26.
- Versão candidata 0.14.0 em Cargo/lock/Dockerfile; `mqtt-client --version` confirmado no WSL.
- Ubuntu26/WSL após bump: fmt, 47 unitários/35 wire, Clippy, build, inventário MIT, 7 contratos de distribuição e integração cruzada com broker0.16.0 PASS. Nenhuma instalação do operador foi alterada.
- Fonte/tag imutável: `7d0c3871c62508f93721426e3b56fdcf8c755540`/`v0.14.0`; [CI](https://github.com/licodevone/xmqr-client/actions/runs/38952170123) PASS, incluindo instalação `.deb` Ubuntu26.
- `.deb` `xmqr-client_0.14.0-1_amd64.deb` e imagem numerada `lepinheiro100/xmqr-client:0.14.0`. Seis anexos da [prerelease](https://github.com/licodevone/xmqr-client/releases/tag/v0.14.0) tiveram nome, tamanho e SHA-256 conferidos contra arquivos locais. Manifesto inclui origem SHA, alvos e hashes.
- Docker Hub remoto `sha256:13316fef3241d04453651f025233d00f74b9f07be24f555d09f6742730dcda3c` conferido; execução por digest imprimiu `mqtt-client 0.14.0`. `latest` preservada.
- Release experimental do subconjunto MQTT5, sem conformidade integral/produção. Upgrade/rollback em [nota](../../docs/releases/0.14.0.md). Instalação WSL do operador não alterada.
