# C137 v1.0 — bridge MQTT 5 QoS 1 com retained preservado

## Entrada

`main` em `76622cb` (client 0.40.0, bridge 0.2.0 publicados), CI C136
`39168330647` PASS. C136 oferece origem MQTT 5 somente com rota QoS 0 e
Retain As Published; MQTT 3.1.1 QoS 0/1/2 permanece compatível. A saída
local `vendor/rumqttc-0.25.1/target/` não rastreada deve ser preservada.

## Objetivo e escopo

Aceitar `source_protocol="mqtt5"` com `route.qos=1`, mantendo destino MQTT
3.1.1. Assinar a origem com QoS 1 e Retain As Published=1, sessão persistente
com expiração finita e ACK manual. A bridge só envia PUBACK na origem após o
PUBACK do destino. Encaminhar o bit RETAIN recebido, inclusive no replay e em
publicações vivas; uma publicação comum não deve substituir o retained no
destino. QoS 2 com origem MQTT 5 continua recusado.

A opção [Retain As Published da seção 3.8.3.1 MQTT 5](https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html)
conserva o bit na entrega de uma assinatura ativa. Isto não cria transação
entre brokers: queda após PUBACK do destino e antes do PUBACK da origem pode
duplicar o encaminhamento. Confirmar compatibilidade concreta com XMQR e
Mosquitto nos dois sentidos da topologia.

Arquivos previstos: `src/bin/mqtt-bridge.rs`, duas fixtures de bridge,
`.github/workflows/ci.yml`, `docs/bridge-one-way.md`, registro C137 e
documentação de evolução após aceite. Não alterar formato do journal QoS 2,
segredos, ACL, systemd, versão ou pacote nesta fatia.

## Invariantes e falhas

- Configuração MQTT 5/QoS 2 falha antes de conectar. Default 3.1.1 e todos
  os modos antigos permanecem. Topologia gerenciada é obrigatória em QoS 1.
- Origem MQTT 5 usa sessão persistente/expiração e ACK manual; em perda de
  sessão, entrega não confirmada deve ser reavaliada como limite, nunca como
  entrega garantida. Mensagem com QoS incorreto, tópico inválido ou payload
  acima do limite falha fechada.
- Destino indisponível ou sem PUBACK impede o PUBACK na origem; não confirmar
  sucesso por enfileiramento local. Um voo por vez, canal limitado e no máximo
  uma mensagem pendente por execução.
- TLS/CA/SAN, usuário, senha em arquivo privado, ACL, prefixos e limites
  anteriores permanecem. Não registrar segredos nem payloads.

## Aceite

Testes reais TLS/mTLS XMQR→XMQR e XMQR↔Mosquitto, nos dois sentidos da
topologia, cobrindo retained anterior, retained vivo, publicação comum,
reconexão, negativos de TLS/ACL, destino indisponível e repetição após queda
no checkpoint entre PUBACKs. Regressões MQTT 3.1.1 QoS 0/1/2 e MQTT 5/QoS 0.
Gates `cargo fmt --check`, `cargo test --locked`, `cargo clippy --all-targets
-D warnings`, `cargo build --locked --bins`, inventário e CI da fonte exata.
Registrar PASS/FAIL/BLOCKED e manter fase 8 global aberta até os demais
contratos serem cumpridos.
