# C45 v1.6 — robustez da sessão e reconexão / fase3 C07

Preparado2026-10-13 antes das fontes. Entrada Gitd9d97d51b88832044c8d2de3786ce4262ee77ec1,
runtime0.8.1; preservar os dois tar/SHA locais0.7.1. Skill xmqr-client-development
lida; AGENTS/README/ORIGIN/P89/plano compartilhado lidos. Pedido explícito
autoriza implementar, validar e publicar por coordenador, agentes paralelos.

## Contrato

- Preservar binário mqtt-client e dependência rumqttc0.25.1 independente do
  broker; TLS/SAN/credenciais0600, limites1024UTF8/4096/65536, MIT/avisos.
- Novo --total-timeout-seconds1..86400, opcional (padrão nenhum), monotônico.
  Começa em session::run, após parse/leitura inicial de payload e credenciais;
  inclui preparoTLS, conexões, protocolo, espera de mensagens/entrada seguinte,
  backoff e DISCONNECT normal. A entrada inicial e senha interativa permanecem
  fora desse prazo; input inicial mantém8s. Não renovar por tráfego/reconexão.
- --timeout-seconds1..300/default8 é prazo de protocolo, por tentativa de
  CONNACK e por SUBACK/PUBLISH/QoS2/DISCONNECT. Assinante idle90s acompanha
  ausência de PUBLISH de aplicação, não PINGRESP; reconexão reinicia idle
  quando o link é restabelecido. Total nunca reinicia. Sem total, compatível.
- Retry0..10 totalporprocesso não resetado por sucesso. Backoff exponencial
  base100..5000ms com jitter limitado80..100% da base; PRNG não criptográfico
  sem nova dependência, seed por processo via RandomState, tests seed fixo.
  Somente transporte retentável; auth/TLS/codec/timeout externo não retentáveis.
- CancelamentoSIGINT/SIGTERM prioritário sobre poll/backoff/input dentro da
  sessão; não criar tentativa extra. DISCONNECT best-effort até1s em link ativo.
  Total expirado encerra sem prazo adicional, código1 e diagnóstico estático.
- Publicação após enfileirar PUBLISH mantém resultado desconhecido em falha,
  sem reenvio. QoS1 pode repetir mensagens legitimamente; não prometer exactly
  once. QoS2 recebido entre PUBLISH e PUBREL mantém fail explícito sem reconexão
  devido clean() da rumqttc. Não inventar persistência local.

## Ownership / arquivos

src/cli.rs, src/session.rs, tests/wire.rs, README, CHANGELOG, docs da robustez,
prompts/README/COBERTURA e registroC45. Sem alterar manifests/lock/Dockerfile/
version: coordenador define marco depois dos gates. Sem services/firewall/
ESP32/redeoperador; fixtures loopback efêmeras e processos próprios.

## Aceite

CLI default/bordas/faltante/duplicado/zero/negativo/overflow e ausência de rede;
deadline total menor que protocolo em CONNACK/SUBACK/PUBACK/idle/backoff;
tráfegoPING não renova total; entrada posterior e DISCONNECT também limitados;
retriesbudgetsemreset, authsemretry, cancelSIGINT/SIGTERM em poll/backoff/input
sem tentativaextra, QoS1duplicata observável e QoS2incerto explícito. Fixtures
ownwire + fmt/testlocked/clippyalltargets-Dwarnings/build LinuxWSLUbuntu26;
interop mTLS/ACL/Mosquitto e broker real por campanhaP91 coordenada.
Registrar PASS/FAIL/NOT_RUN separados; revise promptantesfix se gate falhar.

Fontes oficiais: https://docs.rs/rumqttc/0.25.1/rumqttc/struct.EventLoop.html,
https://github.com/bytebeamio/rumqtt/blob/main/rumqttc/src/eventloop.rs,
https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html.
Context7 consultado para poll/clean: próximo poll reconecta imediatamente,
logo nosso budget/backoff deve precedê-lo. Confirmar versão local pinada.

## Revisão1.1 antes de correção

Primeiro cargo test --locked FAIL de compilação E0618: fixture chama listener()
após bindinghomônimo. Corrigir somente referência qualificada do harness e
reexecutar, não modificar aceites para esconder falha. Inspeção também mostrou
poll de reconexão pós-SUBACK usando idle90s como prazoCONNACK; separar o
protocolo configurado e envolver a espera de SUBACK/idle em deadlineabsoluto,
incluindo backoff, para não renovar espera a cada falha. Novo negativo silencioso
na segunda conexão deve expirar por protocolo1s, não aguardar idle90s.

## Revisão1.2 antes de correção

Primeira suíte completaPASS59 (33unit+26wire). ClippyFAIL clippy::assert_is_empty
em dois assertsnovos de fixture; substituir por assert_eq!/assert_ne!, sem
desabilitar warning. Acrescentar coberturainputposterior/CONNACKreconectado e
CLIinválida; src/main.rs também em escopo apenaspara priorizarsinalna entrada
inicial. NegativoSIGINT/SIGTERM inicial antesdepayload/narede deve confirmar
zeroCONNECT. Senha interativa bloqueante continuafora do prazototal/novocontrato.

## Revisão1.3 — gate real do idle antes de harness

Adicionar scripts/verify_client_robustness.py para um ensaio realisoladoLinux
com80+PINGRESP duranteidle90s (sem novo total), deadlineexterna96s, ports
efêmeras eprocessos próprios. Não simular relógio ou declarar90s pelo helper.
Report JSONsemcredenciais e sha executável. O mesmoharness cobre sinaisantes
da entradainicial: observarhandlersSIGINT/SIGTERM pelo /proc/filho, não sleep
cego, manterstdinaberto e provarzerorede/saída0. Não instalar ou ler serviço real.

## Revisão1.4 antes de correção

SuíteampliadaFAIL1/62: negativosegundoCONNACK expirou, mas erro esperadodiferiu.
Fixture enviavaSUBACK e fechavalink sem confirmarqueclientobservou assinatura;
assim podiaexercitar deadlineSUBACK1s emvezdeidle90s. Sincronizar fechamento
pela linha Assinatura ativa, deixando erroCONNACK e budget assertions intactos.
Não modificarfontes nem aceitar erroqualquer para resolver oFAIL.

## Revisão1.5 antes de correção

Suíte62PASS após sincronização. ClippyFAIL semicolon_if_nothing_returned em
total_expired e too_many_lines102/100 em subscribe. Acrescentar semicolon e
extrair deadline/poll de Retry em método coesivo next_before, sem suprimir
warning ou alterar os budgets. Reexecutar fullgates. HarnessLinux realPASS3
(sinaisentradainicial + idle90comPING); fonte semicolon/helper não mudaaceite.

## Revisão1.6 — teto conjunto antes de reforço

Revisãoprópria identifica sinal/total jáprontos juntos: escolher sinalpoderia
adicionar1s deDISCONNECT alémde total. Limitar cleanupdeSIG ao menor1s/tempo
totalrestante; se totaljáexpirou, apenasdrop. Preservarprioridadesemnova
tentativa. Novo teste cancellation_budget respeita totalpresente/None e
deadlinejápassada. Reexecutar fullgatesapósreforço.
