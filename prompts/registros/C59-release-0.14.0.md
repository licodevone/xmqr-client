# C59 — preparação da release 0.14.0

- Prompt [C59](../clients/59-release-0.14.0.md) criado antes do bump.
- Base funcional `9aa86ae`: [CI](https://github.com/licodevone/xmqr-client/actions/runs/38951630275) PASS, incluindo pacote Ubuntu26.
- Versão candidata 0.14.0 em Cargo/lock/Dockerfile; `mqtt-client --version` confirmado no WSL.
- Ubuntu26/WSL após bump: fmt, 47 unitários/35 wire, Clippy, build, inventário MIT, 7 contratos de distribuição e integração cruzada com broker0.16.0 PASS. Nenhuma instalação do operador foi alterada.
- Commit de release, CI desse SHA, tag, assets e publicação: registrar após execução; não presumir PASS.
