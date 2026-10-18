# C137 — origem MQTT 5/QoS 1 na bridge

Prompt [C137 v1.0](../clients/137-bridge-mqtt5-qos1-retain.md) enviado em
`3b187aa` antes da implementação. Base: client 0.40.0/bridge 0.2.0,
`source_protocol="mqtt5"` C136 somente QoS 0. A saída local não rastreada
`vendor/rumqttc-0.25.1/target/` foi preservada.

## Comportamento

Origem MQTT 5/QoS 1 usa assinatura com Retain As Published, sessão com
expiração de 24 horas e ACK manual depois do PUBACK do destino MQTT 3.1.1.
QoS 2 com origem MQTT 5 segue recusado. O modo MQTT 3.1.1 por default,
journal QoS 2 e guardião de topologia gerenciada permanecem.

Não existe transação entre brokers nem journal local da origem MQTT 5/QoS 1.
Queda após PUBACK do destino e antes do PUBACK da origem pode duplicar; perda
ou expiração da sessão de origem pode descartar pendências. Os ensaios não
demonstram perda de energia ou prontidão de produção. Esta opção ainda não
integra uma release.

## Evidência local

- Primeiro `cargo check --locked --offline --bin mqtt-bridge -j1`: PASS.
- `cargo test --locked --offline -j1`: PASS. Primeira execução de Clippy:
  FAIL `too_many_lines` na função MQTT 5. Extração do encaminhamento QoS 1
  em função própria; `cargo clippy --locked --offline --all-targets -j1
  -- -Dwarnings`, build dos binários e inventário MIT: PASS.
- Dois brokers XMQR TLS/mTLS MQTT 5/QoS 1: PASS, incluindo retained anterior,
  retained vivo, publicação comum, ACL, CA/SAN/senha, reconexão, manifesto,
  checkpoint de crash após PUBACK do destino e redelivery.
- XMQR→Mosquitto 2.0.22 e Mosquitto→XMQR MQTT 5/QoS 1: PASS nos dois sentidos.
- Regressões C136 MQTT 5/QoS 0, MQTT 3.1.1 QoS 1 e QoS 2 (dois checkpoints
  de crash) com XMQR e Mosquitto: PASS.

CI da fonte exata pendente antes de marcar esta fatia no plano. T02/fase 8
continua aberta para propriedades MQTT 5, ciclos externos, semântica ampliada
e distribuição/homologação da nova fatia.
