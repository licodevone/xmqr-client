# C136 v1.0 — bridge MQTT 5 com Retain As Published

## Entrada

`main` em `f6ecde83b4f9d9f6d1af521e55c3309773265f03`, client 0.40.0 e
bridge 0.2.0 publicados. C125–C135 oferecem origem/destino MQTT 3.1.1,
QoS 0/1/2, TLS/mTLS, jornal QoS 2 e guardião de topologia gerenciada.
`vendor/rumqttc-0.25.1/target/` é saída local não rastreada; preservar.
Fase 8/T02 continua aberta. Em MQTT 3.1.1, uma publicação retained viva
entregue a uma assinatura ativa normalmente chega com RETAIN=0; a bridge
não pode inferir o valor original desse bit.

## Objetivo e escopo

Acrescentar seleção opt-in de protocolo MQTT 5 na **origem** da bridge para
`route.qos=0`, assinando com a opção `Retain As Published=1`. Encaminhar ao
destino MQTT 3.1.1 o RETAIN recebido: replay retained e publicação retained
viva ficam retidos; publicação comum não deve ser retida. Manter o modo
MQTT 3.1.1 atual como default. Recusar combinação MQTT 5 + QoS 1/2 nesta
fatia, sem enfraquecer o journal, ACK e restauração existentes; esses modos
exigem contrato e ensaios próprios antes de serem oferecidos.

Arquivos previstos: `src/bin/mqtt-bridge.rs`, fixture real de dois brokers,
guia `docs/bridge-one-way.md`, registro C136. Não alterar formato do jornal
QoS 2, senha, ACL, unidade systemd, versão ou pacote nesta fatia.

## Invariantes e falhas

- A nova assinatura deve transmitir RAP=1 no wire; confirmar com broker XMQR
  e outra implementação quando disponível. Não fabricar RETAIN=1 para
  mensagem comum e não usar heurística de payload/tópico.
- Prefixos, tamanho de tópico/payload, ACL, TLS/CA/SAN, limite de reconexão
  e descarte QoS 0 com destino indisponível continuam válidos. Tópico MQTT 5
  vazio/sem mapeamento é recusado ou descartado de forma delimitada, nunca
  publicado em tópico incorreto.
- Nada de segredo/payload em logs ou relatórios; recursos próprios, temporários
  e limpeza explícita nos testes. `mqtt-bridge --version` permanece livre de
  configuração.
- Esta etapa não promete transação entre brokers, retained vivo QoS 1/2,
  propriedades MQTT 5 no destino, prevenção de ciclos externos ou T02 aceito.

## Aceite

Unitários para seleção/default, incompatibilidade QoS 1/2, tópico e política
de bit RETAIN. Integração TLS/mTLS com origem XMQR MQTT 5 e destino XMQR
MQTT 3.1.1: retained anterior, retained publicado após assinatura, publicação
normal, reinício/reconexão e negativos de credencial/ACL/CA/SAN. Quando
possível, repetir XMQR↔Mosquitto. Regressão C125–C135, `cargo fmt --check`,
`cargo test --locked`, `cargo clippy --all-targets -D warnings`, build e CI
da fonte pública exata. Registrar PASS/FAIL/BLOCKED sem converter um modo
parcial em aceite global da fase 8.

## Revisão v1.1 — ordenar assinatura após destino pronto

Primeiro ensaio real de dois XMQR: compilação e seis unitários PASS, mas
subscriber retained no destino encerrou por timeout antes dos casos vivos.
Origem assinava assim que recebia CONNACK, enquanto a conexão do destino
era iniciada em paralelo; replay QoS 0 podia chegar no intervalo e ser
descartado. Esta é a hipótese, não um diagnóstico confirmado por log privado.
Antes da correção, preservar o FAIL. No modo novo, aguardar o destino ficar
conectado com prazo finito antes de SUBSCRIBE MQTT 5; se não ficar pronto,
falhar sem afirmar replay. Não aumentar fila nem mudar ACK dos modos QoS 1/2.
Repetir a fixture, incluindo negativo de destino indisponível e regressões.
