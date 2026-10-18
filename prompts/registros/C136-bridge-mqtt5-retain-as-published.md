# C136 — origem MQTT 5 e Retain As Published na bridge

Prompt [C136 v1.1](../clients/136-bridge-mqtt5-retain-as-published.md) foi
criado e enviado antes da alteração funcional. Base client0.40.0
`f6ecde83b4f9d9f6d1af521e55c3309773265f03`; saída local ignorada
`vendor/rumqttc-0.25.1/target/` preservada.

## Entrega em validação

Opt-in `source_protocol="mqtt5"` aceita somente rota QoS 0; default MQTT
3.1.1 e journal QoS 2 anteriores permanecem. A assinatura MQTT 5 usa
`Retain As Published=1`; o bit RETAIN recebido é enviado ao destino 3.1.1,
sem fabricar retained para publicação comum. Antes da assinatura, o modo
novo aguarda conexão do destino com limite e observa falha do task de
destino; configuração inválida ou SUBACK negado falham fechados.

## Evidência e falhas preservadas

- `cargo check --locked --offline --bin mqtt-bridge -j1`: PASS; após fmt,
  `cargo test --locked --offline --bin mqtt-bridge -j1`: 6/6 PASS;
  build debug PASS.
- Primeiro ensaio real XMQR→XMQR: FAIL por timeout no subscriber retained
  antes de chegar aos casos vivos. Hipótese de corrida entre assinatura da
  origem e conexão do destino; C136 v1.1 precedeu a correção.
- Retry inicial: FAIL no negativo CA porque o novo prazo de espera de 20s
  excedia os 15s do harness. Alteração seguinte observa a falha do task de
  destino e termina prontamente, sem ampliar o timeout do negativo.
- Ensaio real XMQR→XMQR TLS/mTLS com replay retained, retained vivo,
  publicação normal, ACL/CA/SAN/senha e reconexão: PASS local.
- Interoperabilidade MQTT 5 com Mosquitto 2.0.22 em ambas direções
  XMQR→Mosquitto e Mosquitto→XMQR, replay/live retained, publicação comum
  e reconexão: PASS local.

- Fullsuite `cargo test --locked --offline -j1`: PASS. Primeira execução de
  Clippy apontou `cast_possible_truncation` na conversão do limite de bytes;
  `u32::try_from` corrigiu sem `allow` e repetição de fmt, Clippy
  `--all-targets -Dwarnings`, build de todos os binários e inventário MIT:
  PASS.
- Regressões reais antigas XMQR↔XMQR e XMQR↔Mosquitto em QoS 0, 1 e 2,
  incluindo checkpoints de crash QoS 2: PASS nas seis combinações.

CI exata ainda pendente. A opção C136 não integra a release0.40.0;
T02/fase8 segue aberta para QoS1/2 MQTT5,
propriedades, ciclos externos e semântica entre brokers. Testes finitos não
comprovam transação distribuída, perda de energia ou produção.
