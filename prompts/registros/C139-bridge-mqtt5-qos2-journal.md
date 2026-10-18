# C139 — origem MQTT 5/QoS 2 na bridge com journal isolado

Prompt: [139-bridge-mqtt5-qos2-journal.md](../clients/139-bridge-mqtt5-qos2-journal.md), publicado antes da funcionalidade no commit `b789481`.

## Resultado da fonte

`source_protocol="mqtt5"` aceita `route.qos=2` com destino MQTT 3.1.1. A origem usa sessão persistente, assinatura QoS 2 com Retain As Published e ACK manual. Após PUBCOMP do destino, a bridge registra a obrigação no journal antes de PUBREC. Depois de PUBREL, registra intenção de PUBCOMP antes de confirmar a origem. A identidade do journal inclui o protocolo, preservando a separação de MQTT 3.1.1. Uma sessão de origem ausente com registros pendentes causa falha fechada.

O journal conserva tombstones de conclusão até reutilização do Packet Identifier ou liberação de espaço. A fixture verifica mensagens ativas separadamente desses tombstones. O limite continua 32 registros; registros ativos não são descartados. O Mosquitto de teste usa persistência para que o reinício de origem preserve a sessão QoS 2.

## Evidência local

- `cargo fmt --all -- --check`, `cargo clippy --locked --offline --all-targets -j1 -- -D warnings`, `cargo test --locked --offline -j1`: PASS.
- `verify_bridge_brokers.py --broker-root ../xmqr --mqtt5-retain --qos2`: PASS, com retained, ACL/TLS, dois checkpoints de queda, destino reiniciado e sessão da origem perdida.
- `verify_bridge_mosquitto.py --broker-root ../xmqr --mqtt5-retain --qos2`: PASS nas direções XMQR→Mosquitto e Mosquitto→XMQR. A primeira tentativa na direção Mosquitto→XMQR demonstrou ausência de persistência no container da fixture; após correção da fixture, ambas passaram. Uma tentativa anterior teve falha de publicação de porta do Docker, superada na repetição.

CI da fonte exata: pendente até publicação e conclusão. O binário client/bridge 0.41.0 já publicado **não** inclui C139. Sem tag/release nesta fatia.

## Limites

A confirmação QoS 2 vale por perna MQTT, não constitui transação distribuída. Uma queda após PUBCOMP do destino e antes da escrita local pode duplicar o envio entre brokers. Propriedades MQTT 5 não atravessam a bridge. Perda da sessão de origem exige intervenção conservando o journal. O checkpoint de debug cobre duas janelas de crash; não é teste de corte real de energia. T02/fase 8 continua aberto.
