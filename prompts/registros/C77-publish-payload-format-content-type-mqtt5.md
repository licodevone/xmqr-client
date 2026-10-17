# C77 — formato e tipo de conteúdo no publisher MQTT 5

- [Prompt C77](../clients/77-publish-payload-format-content-type-mqtt5.md) criado antes do código. Base 0.19.0 aceita; implementação, testes, CI e aceite pendentes. Fase 6C aberta.
- Parser de `--payload-format`/`--content-type`, wire com ambas e Message Expiry, e rejeição de arquivo binário antes da conexão PASS. Integração TCP com broker P141 QoS0/1/2 PASS. Gate local: CLI 54 PASS, wire 50 PASS, fmt, Clippy `-D warnings`, build release e inventário de licenças PASS. CI e aceite pendentes.
- Interoperabilidade externa: client publicou PFI=1/Content Type=`text/plain` em QoS1 a broker Mosquitto 2 em contêiner descartável; `mosquitto_sub` MQTT 5 recebeu payload e PUBACK confirmou (`MOSQUITTO_BROKER_INTEROP_PASS`). O primeiro comando do ensaio falhou por expansão de variáveis no shell PowerShell; repetido por script temporário, passou, contêiner e scripts temporários removidos. Nenhuma instalação no WSL do operador.
