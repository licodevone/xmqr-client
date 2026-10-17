# C84 — Retain Handling MQTT 5 no client

- [Prompt C84](../clients/84-retain-handling-mqtt5.md) criado antes do código. Base C83 aceita e broker P147 em validação; fase 6D global aberta.
- Implementação: flag `--retain-handling 0|1|2` apenas para `sub` MQTT 5; opção aplicada a cada filtro, padrão 0. Validação ocorre antes da rede. `--ignore-retained` e `--retained-only` continuam filtros locais independentes; sem alteração do perfil TOML v1.
- Gates locais: 58 testes CLI e 58 wire PASS, fmt, Clippy `-D warnings`, build release e licenças PASS. [TCP real P147/C84](../../../xmqr/scripts/verify_mqtt5_retain_handling.py) PASS com broker XMQR e Mosquitto 2.0.22, modos 0/1/2 e sessão persistente/restart; imagem externa `sha256:199ea8ef2e35ec2b1b37e59cfd1dbae538ed4dfa4a2251a121a52215a6248a21`, contêiner removido. Regressões C80/C83 PASS. Sem ensaio ESP32.
- Fonte `068484b67077a432779f00f5cd155633eeb901bd` em `main`; [CI completa PASS](https://github.com/licodevone/xmqr-client/actions/runs/39003795048), incluindo pacote Ubuntu26. C84 aceito somente para Retain Handling do assinante; sem release isolada nem aceite global da fase 6D.
