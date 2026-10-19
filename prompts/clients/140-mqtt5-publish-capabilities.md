# C140 v1.1 — capacidades de publicação MQTT 5

Revisão após FAIL de compilação do teste: ConnAckProperties não implementa
Default nessa dependência. Construir CONNACK do teste por bytes independentes
e decodificação pública. Não alterar produção para acomodar teste.

v1.2: Clippy ainda encontrou run_inner com 102 linhas após validação da
sessão durável. Extrair a validação pré-existente de alias em helper separado,
mantendo a comparação/erro exatos; não suprimir a regra de tamanho.

## Entrada e autorização

Pedido do usuário em 2026-10-18: solucionar as cinco falhas obrigatórias da
auditoria P232, com prompt antes do código, testes e publicação após aceite.
Client `main` em `e03c5a46cfb33666896eb710892cf46e3c40ee38`, versão 0.41.0;
C139 já está integrado. Preservar `vendor/rumqttc-0.25.1/target/` não rastreado.
Broker 0.43.0 e admin 0.11.0 não precisam de mudanças de runtime nesta correção.

## Objetivo e escopo

Em `src/v5_session.rs`, impedir PUBLISH acima de Maximum QoS anunciado e
RETAIN=1 quando Retain Available=0 (MQTT-3.2.2-11 e MQTT-3.2.2-14).
Ausência equivale a QoS máximo 2 e retained disponível. Não reduzir QoS
silenciosamente. Recusar valores inválidos dessas propriedades com diagnóstico
fixo, sem reproduzir campos remotos. Checar antes de enfileirar publicação
normal/repetida/outbox; uma continuação PUBREL já persistida não é um novo
PUBLISH. Não apagar journal em recusa de capacidade.

## Aceite e invariantes

Reproduzir RED com binário publicado 0.41.0 e servidor sintético loopback;
depois GREEN para cada restrição, limites permitidos, propriedades ausentes,
QoS 2, repetição e outbox. Testes unitários e wire sem dados reais. Preservar
TLS/SAN, MQTT 3.1.1 padrão, limites, licenças e estados anteriores. Registrar
PASS/FAIL/BLOCKED em registro C140–C142; após FAIL revisar prompt antes de
correção. Rodar fmt/test/Clippy/build e inventário antes de publicação conjunta.

Referência: [OASIS MQTT 5.0 OS](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html),
§3.2.2.3.4/5. As quatro recomendações de validação recebida da P232 não fazem
parte das cinco violações obrigatórias e não serão declaradas corrigidas.
