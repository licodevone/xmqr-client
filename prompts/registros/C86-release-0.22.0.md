# C86 — prerelease experimental client 0.22.0

- [Prompt C86](../clients/86-release-0.22.0.md) criado antes do bump. C83–C85 e P148 aceitos com CI PASS.
- Candidato: Cargo/lock e Dockerfile 0.22.0, changelog, README e nota de release atualizados. O client não guarda fluxo QoS MQTT 5 para rollback; perfis/segredos permanecem externos.
- Gates locais do candidato PASS: 59 testes CLI e 59 wire, fmt, Clippy `-D warnings`, build release, inventário de licenças e sete contratos de distribuição. O ensaio P148/C85 com broker XMQR em desenvolvimento passou após sincronização de ACK antes do restart; C85 já passou com Mosquitto 2.0.22.
- Fonte `c7726d1c1ffa2b7683b0de5ab87a84144ec27c57`, [CI PASS](https://github.com/licodevone/xmqr-client/actions/runs/39005767449). Interoperabilidade P148/C85 com broker 0.24.0 passou para QoS0/1/2, reinício, No Local e RAP.
- Pacote Ubuntu 26.04 amd64 SHA-256 `5dac2e7a47c15cb9f2317e7bbfe9e8e109a0558d0d3fae96db5578b10dc1c996`; ciclo descartável 0.23.0/0.21.0 → 0.24.0/0.22.0 → anterior → purge `PACKAGE_LIFECYCLE_PASS`.
- Tag anotada `v0.22.0` aponta à fonte exata. [Prerelease](https://github.com/licodevone/xmqr-client/releases/tag/v0.22.0) com seis anexos; tamanhos e SHA-256 remotos iguais aos locais. Docker Hub `lepinheiro100/xmqr-client:0.22.0`, digest `sha256:e244e6de4167f8d40d64fdbe4b8379e8a352fc18a496a81db887f985bc795ac0`, executado por digest sem rede. `latest` intocada; WSL do operador não alterada.
- C86 aceito como distribuição experimental; fases 6A–6D e 7–9 seguem abertas.
