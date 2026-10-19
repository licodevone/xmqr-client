# C144 v1.0 — quatro recomendações de recepção MQTT 5

Data: 2026-10-18. Pedido: «Resolva essas recomendações» após a verificação
dos cinco MUST de P232. A execução autoriza prompt, correções e validação;
não solicita instalação, publicação, mudança de versão ou nova fase.

Entrada: HEAD `1e73717d37649b2e423b72fd823245991f3fb7f2`, client 0.42.0,
bridge 0.4.0. Único conteúdo local preexistente não rastreado:
`vendor/rumqttc-0.25.1/target/`; preservar. C140–C143 já aceitos/publicados.
Ler AGENTS, README, ORIGIN e skills development/tests/security-review.

Objetivo: rejeitar NUL em tópico e em User Property, Variable Byte Integer
redundante e flags reservadas inválidas de PUBACK. São recomendações de
validação do receptor Client em §4.13.1 (SHOULD), não novos MUST de fechamento
do Server nem declaração de conformidade integral. Fonte:
https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html
Seções de representação 1.5.4/1.5.5 e flags 2.1.3 definem as entradas válidas.

Escopo: codec MQTT 5 da cópia rumqttc fixada, em
`vendor/rumqttc-0.25.1/src/v5/mqttbytes/v5/{mod,publish}.rs`, nota de patch,
`tests/mqtt5_receive_validation.rs`, fixture CLI autocontida em `scripts/`,
CI, README e registro `prompts/registros/C144-mqtt5-receive-validation.md`.
Validar strings UTF-8 MQTT no leitor compartilhado e o tópico binário antes
de expor PUBLISH; não aplicar NUL/UTF-8 a payload ou Correlation Data.
Validar VBI mínimo no leitor comum (Remaining Length, comprimentos de
propriedades e Subscription Identifier). Preservar fragmentação, limites e
representação canônica nas quatro larguras. Validar flags fixas em todos os
tipos MQTT 5 que as reservam, inclusive PUBREL/SUBSCRIBE/UNSUBSCRIBE=0x2;
PUBLISH mantém seus bits de significado próprio. Rejeitar header inválido
antes de aguardar corpo. Não alterar codec MQTT 3.1.1, persistência, TLS,
ACL, versões ou contratos das licenças MIT/Apache-2.0.

Critérios de aceite: testes de bytes independentes do encoder, negativos e
controles Unicode/BOM/string vazia, payload binário com NUL, propriedades
binárias, flags, VBI mínimo/redundante/truncado, fragmentação/concatenação.
Provar RED antes do patch; manter resultados anteriores. Fixture CLI deve
observar erro rápido e ausência de entrega/ACK de sucesso, além de controles
canônicos. P232 original sem alteração deve passar os 12 cenários, e os 42
casos de C140–C142 devem continuar PASS. Gates no Ubuntu26/WSL: fmt --check,
test --locked, clippy --all-targets -D warnings, build --locked --bins e
inventário de licenças. Exercitar journal inbound e compatibilidade existentes
porque o decoder é compartilhado com ACK manual/bridge. Relatórios novos,
hashes do binário e fixtures, prazos/cleanup, loopback e estado temporário.

Migração: sem formato novo ou ação sobre estado. Servidor que envia os quatro
tipos inválidos passa a provocar encerramento com erro antes de entrega ou
confirmação; corrigir o servidor em vez de ignorar a validação. Não promover
histórico P232 a PASS retroativo. Registrar PASS/FAIL/BLOCKED e limites reais.

## v1.1 — reprodução antes da correção

P232 original contra o debug 0.42.0: 8 PASS/4 FAIL, reproduzindo as quatro
recomendações. Teste novo: 2 PASS/5 FAIL, esperados na fase RED (NUL tópico,
NUL propriedade, VBI externo/interno e flags). Nenhum erro de compilação.
Os controles de Unicode/binário e fragmentação já passam. Executar agora o
patch nos leitores comuns, mantendo as asserções e o oráculo P232 intactos.
