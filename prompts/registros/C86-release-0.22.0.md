# C86 — prerelease experimental client 0.22.0

- [Prompt C86](../clients/86-release-0.22.0.md) criado antes do bump. C83–C85 aceitos; P148 broker ainda em CI.
- Candidato: Cargo/lock e Dockerfile 0.22.0, changelog, README e nota de release atualizados. O client não guarda fluxo QoS MQTT 5 para rollback; perfis/segredos permanecem externos.
- Gates locais do candidato PASS: 59 testes CLI e 59 wire, fmt, Clippy `-D warnings`, build release, inventário de licenças e sete contratos de distribuição. O ensaio P148/C85 com broker XMQR em desenvolvimento passou após sincronização de ACK antes do restart; C85 já passou com Mosquitto 2.0.22.
- Commit exato, CI, pacote/imagem, ciclo de instalação, tag e publicação: **pendentes**. Não declarar 0.22.0 publicada até conferência remota.
