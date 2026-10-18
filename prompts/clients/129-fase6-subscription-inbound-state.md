# C129 v1.10 — journal inbound da assinatura persistente MQTT 5

18/10/2026, preparado antes do código. Pedido: fechar M03/6B do plano de
paridade e a pendência explícita do prompt C110, coordenado pelo broker P227.
Entrada: clone local próprio limpo b9f46cb49646335bcb93ffa769eb2042796524b7,
client publicado0.36.0, broker0.42.0/admin0.10.0. C128/bridge em edição no
repositório original é alheio; não importar/reverter/commitar seus arquivos.

## Escopo e decisões

Ownership: src/outbox.rs, src/v5_session.rs, tests/wire.rs, novos testes
dedicados e vendor/rumqttc-0.25.1/src/v5/{state.rs,client.rs}, mais
vendor/rumqttc-0.25.1/XMQR-PATCHES.md. Sem dependência nova, Cargo/Docker/CI,
README/CLI/bridge alterados. --subscription-state-dir existente é o opt-in,
Linux/WSL, mesmo SubscriptionSpec/destino/identidade/filtros e SessionPresent1.
Preservar outbox de publicação C105/C107, MQTT3.1.1 e modo sem manifesto.
IDs novos/flags/nomes de usuário não serão presumidos. O coordenador integra
docs globais e distribuição depois dos gates; não editar repositório original.

Publisher continua serial, quota inicial1 é permitida por MQTT4.9 e não exige
pipeline por performance. Backlog/limites bilaterais do broker serão testados
separadamente, sem retirar requisito existente. Journal inbound contém no
máximo32 registros e65536 bytes serializados, incluindo metadados, no diretório
0700 e arquivos0600 com proprietário/lock/no-follow/hash/fsync/rename existentes.
Quota cheia ou armazenamento incerto fecha sem ACK, preservando a obrigação
do broker. Não aumentar silenciosamente quotas ou prometer produção.

## Estados e falhas

Received: pacote QoS1/2 resolvido (sem alias transitório), payload binário,
metadados/propriedades e identificador validados e duráveis antes da saída e
de qualquer ACK. OutputReady: depois de saída/flush ou filtro, persistir a
transição antes do PUBACK/PUBREC. QoS2 segue AwaitPubRel; ao receber PUBREL,
persistir PubCompPending antes de enfileirar PUBCOMP. Limpar registro após
evento Outgoing do ACK terminal; erro de limpeza permanece explícito.
Reabrir Received pode repetir stdout: queda entre flush e commit não é uma
transação com o consumidor. Não alegar exatamente uma vez de aplicação.
Erro de saída não confirma mensagem; payload continua recuperável no journal.
Duplicatas da mesma obrigação não repetem saída de uma fase já confirmada;
identificador reusado após conclusão inicia nova publicação, inclusive DUP.
Sem SessionPresent ou especificação divergente, preservar todos os arquivos
e falhar antes de saída/replay/ACK. Sem segredos ou payloads nos diagnósticos.

Fork: modo explícito de incoming QoS2 controlado pelo journal, ativado apenas
na assinatura opt-in. Evitar PUBCOMP automático e rejeição prematura de PUBREL
após restart por bitset transitório vazio. Encaminhar PUBREL ao caller que
valida journal, completa duravelmente e enfileira PUBCOMP manual; desconhecido
recebe motivo0x92 sem saída falsa. Preservar comportamento padrão e todos
copyrights/textos Apache-2.0 originais; registrar patch sem relicenciamento.

## Regressões e aceite finito

Antes do fix, executar reprodução dirigida no ELF antigo da assinatura que
crasha após PUBREC e retoma com SessionPresent1/PUBREL: exigir conclusão,
continuidade e ausência de saída duplicada nesse caso. Manter RED real.
Unitários de roundtrip/fases/checksum/limite32/64KiB/lock/symlink/permissões;
fixtures TCP de QoS1/2 e recuperação, ACK após commit, saída falha, filtros,
crash após PUBREC/antesPUBREL, deadline enquanto aguarda PUBREL, PUBREL repetido,
SessionPresent0, declaração divergente e arquivo corrompido sem replay/ACK.
Repetir C105/C107/C108/C110 e WSS quando pertinentes em recursos sintéticos.
Não simular perda de energia por SIGKILL nem usar serviços/estado do operador.

Gates locked fmt/test/Clippy/build/inventário, Windowscheck proporcional,
fonte exata/hash/ambiente e resultados reais em registro C129. TMPDIR=/var/tmp,
cache privado -j1; não copiar target/cache inteiro nem instalar ferramentas.
Prompt revisado antes de correções após FAIL. Sem tag/push/publicação pelo
agente; commits locais autorizados para isolar a contribuição. Coordenador
decide integração e release após aceite e regressões finais.

Norma primária consultada: OASIS MQTT5 §§3.2.2,4.1,4.3.2/4.3.3,4.4,4.9:
https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html

## Revisão v1.1 — RED real antes do fix

Fixture TCP sintética executada no ELF debug existente, copiado isoladamente:
SHA415222bdc6855eaeba6130829225c82df0d740f7015159f4c22e82e93c3d1a8e.
CONNECT/SUBACK, saída e PUBREC ocorreram; SIGKILL após PUBREC, nova conexão
com SessionPresent1 e PUBREL resultou em EOF em vez de PUBCOMP. Exit1,
0.5474s; relatório dist/c129-inbound-baseline.json preservado. HarnessSHA
5d024da41b152fd22399b116c37f35de59b223c2cfdbdeaa6a808078153a1896.
Essa falha esperada reproduz a ausência do journal/bitset durável, não uma
falha de fixture. Iniciar agora a correção delimitada descrita acima.

## Revisão v1.2 — primeiro gate de formatação

fmt --check retornou1 por estilo/imports dos novos blocos, sem compilação ou
teste funcional novo. Corrigir mecanicamente por rustfmt apenas arquivos
próprios/patch do fork; não interpretar formatação como falha de protocolo.
Journal serializa PUBLISH MQTT5 canônico com tópico resolvido e sem alias/DUP
transitórios, preservando metadados binários/ordem. Hash de identidade exclui
somente intervalo restante de expiração, que pode diminuir num replay; hash
do documento cobre todos os bytes. Testes acrescentam validação da fronteira
32records/64KiB e rejeição atômica antes de ACK quando commit não ocorre.

## Revisão v1.3 — continuidade QoS2 e fronteiras do event loop

RED adicional no mesmo ELF antigo, antes da correção: assinatura MQTT5 QoS2
sem count/manifesto completou o primeiro PUBCOMP, mas encerrou antes de uma
segunda publicação. dist/c129-default-baseline.json, exit1/0.3489s,
harnessSHA73902cf7ae0d5eb770c7211793ebcdb56f36e7a15f60e45ca026eeb55a49b230.
O coordenador autorizou aceitar IncomingPubRel já validado/autocompletado pela
biblioteca no modo padrão. PUBREL desconhecido continua rejeitado pelo estado
padrão; não ignorar PUBREL no journal nem alterar autoACK fora do opt-in.
Exigir duas entregas/conclusões contínuas, negativo desconhecido e cancel/idle.

Antes de implementar: obrigação inbound ativa terá deadline absoluto, não
renovado por PINGRESP/duplicatas; expirado, fechar preservando estado. --count
só cancela após ACKs terminais pendentes; mensagens excedentes não ganham saída.
Fila manual opt-in limitada a32, com enqueue não bloqueante e erro explícito:
caller/event-loop único não pode aguardar a própria fila cheia de ACKs. Modo
sem manifesto mantém canal8. Testar lote32 e timeout sem perda silenciosa.
Registros terminais ReadyAck/PubCompPending podem ser coletados ao admitir
outro PUBLISH: saída/obrigação de aplicação já durável, QoS1 admite duplicata e
PUBREL sem registro responde0x92. Não coletar Received/AwaitPubRel; manter
limite32/64KiB, integridade e commit atômico. Acrescentar negativos/quotas e
metadados binários/expiração ao conjunto de testes antes de alegar aceite.

## Revisão v1.4 — build do binário, sem dependência nova

Primeiro build real do binário retornou1 (E0432): bytes é dependência de teste,
não de produção. Os cinco unitários prévios PASS não comprovavam esse build.
Preservar Cargo/lock e a decisão sem dependência nova; usar inferência de tipos
dos métodos públicos write/read já oferecidos por rumqttc para seus buffers.
Não exportar novo módulo nem incorporar serializer paralelo por esse motivo.

## Revisão v1.5 — FAIL de fixture e contrato count existente

dist/c129-inbound-candidate-2.json preserva15PASS/1FAIL; o timeout count foi
no accept antes de CONNECT. Diagnóstico confirmou que C108/CLI já proíbe
--subscription-state-dir junto com --count. Não é defeito do ACK manual e não
autoriza novo modo CLI. Corrigir fixture para exigir recusa estática sem rede
nem journal; retirar defesa count-manual inalcançável/parametrização extra do
runtime. Count padrão QoS2 mantém teste wire anterior (handshake antes de
UNSUBSCRIBE/DISCONNECT). Corrige a hipótese da v1.3, sem relaxar defaults.
Os7 unitários inbound e2 novos wire passaram; os15 casos TCP válidos também.

## Revisão v1.6 — gate integral e Clippy

cargo test --locked --offline integral PASS:2+103+1+68 testes,0 ignorados,
incluindo MQTT3.1.1/5, contagem/cancelamento existentes e os novos negativos.
fmt/fork rustfmt PASS. Primeiro Clippy all-targets retornou1 pela recomendação
assert_is_empty no novo teste de PUBREL desconhecido (wire.rs1991). Ajustar a
asserção para mostrar bytes em falha, sem silenciar lint ou mudar protocolo.
Reexecutar Clippy e inventário após esse ajuste, registrar avisos adicionais
antes de qualquer nova correção não mecânica que se revelar necessária.

## Revisão v1.7 — estilo dos buffers e fronteira de inicialização

Segundo Clippy revelou default_trait_access nos dois buffers inferidos e
too_many_lines em run_inner (103/100) após configurar o modo manual. Construir
os buffers vazios por conversão de slice, ainda usando tipos inferidos da API
existente, e extrair a construção/configuração do event loop para helper
privado. Não adicionar dependência, export ou allow para esconder esses lints.
Os testes funcionais continuam PASS no checkpoint anterior; repetir gates.

## Revisão v1.8 — contrato do agregador de compatibilidade

Primeira execução do agregador novo: C105/C107/C108/C110/C123WSS PASS;
verify_wss_client exit0, mas agregador exigiu marker inexistente
C116_WSS_CLIENT_PASS. Relatório dist/c129-compatibility-1.json FAIL preservado,
sem imputar falha ao runtime nem transformar exit0 sozinho em aceite. Corrigir
para exigir os cinco tokens reais C114_WSS_MTLS_FIXTURE_PASS,
C116_PROFILE_FIXTURE_PASS, C119_WSS_QOS1_OUTBOX_PASS,
C120_WSS_QOS1_RECOVERY_PASS e C122_WSS_QOS2_OUTBOX_PASS. Acrescentar testes
de marker parcial/negativo/código nãozero; repetir todos os seis harnesses.

## Revisão v1.9 — revisão negativa de identificador ainda vivo

Inspeção final encontrou risco em receive_inbound: QoS1 Received, cuja saída
falhou e ainda não foi confirmada, poderia ser substituído por PUBLISH de
conteúdo diferente com o mesmo identificador. QoS1 permite duplicata, não
autoriza apagar uma obrigação não entregue por reutilização protocolar
inválida. Adicionar primeiro RED unitário de identidade divergente QoS1,
preservando bytes/Received. Se reproduzir, rejeitar conflito em qualquer
Received vivo; repetição idêntica continua permitida, fases terminais
continuam reutilizáveis. Não exigir exatamente uma vez nem ampliar quotas.

## Revisão v1.10 — RED confirmado antes da guarda QoS1

inbound_qos1_undelivered_record_cannot_be_overwritten_by_conflicting_identifier
retornou1: receive_inbound aceitou outro conteúdo para QoS1 Received com ID7,
0PASS/1FAIL,0.06s. Corrigir agora a guarda mínima descrita na v1.9 para
Received de qualquer QoS e preservar duplicatas idênticas/ID após conclusão.
Acrescentar prova TCP a partir de saída falha sem ACK, SP1 e PUBLISH conflitante:
recusar sem saída/ACK e sem substituir o arquivo durável. Repetir gates.
