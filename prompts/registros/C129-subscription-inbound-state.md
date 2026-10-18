# C129 — journal inbound MQTT 5, validação local

18/10/2026. Execução autorizada para fechar a obrigação explícita C110/M03,
coordenada pelo broker P227. Este registro não declara a fase6 global aceita.

## Proveniência e isolamento

Clone próprio criado com --no-hardlinks, checkout destacado da base
b9f46cb49646335bcb93ffa769eb2042796524b7, client0.36.0. Commits locais:

- 31d28865918f38cde3206eac99a1ae2be47fd349: prompt antes de código.
- 0456158: reprodução RED TCP antes da implementação.
- 169c03a6f4d080d5415a1fa35f401e72e51437d7: implementação/testes congelados.

O repositório original e alterações C128/bridge/distribuição ficaram intocados.
Cargo.toml/lock, README, CI, Docker, CLI, mqtt-bridge.rs e LICENSE do fork
continuam iguais à base. Nenhum push/tag/release, instalação, serviço,
firewall, placa ou estado do operador foi usado/modificado pelo agente.

## Resultado implementado

O opt-in Linux/WSL existente --subscription-state-dir continua exigindo
assinatura persistente, ID fixo, especificação idêntica e Session Present1
para retomada de manifesto ativo. Continua incompatível com --count.
Não há nova flag nem pipeline do publisher; publicação continua serial1.

subscription-inbound.json contém até32 registros e65536 bytes de documento
JSON, incluindo payload binário e propriedades. O limite de bytes pode
recusar antes de32 mensagens grandes. Diretório0700, arquivo0600, dono,
lock exclusivo, no-follow, checksum, fsync do arquivo e diretório e rename
reutilizam o modelo existente. Quota, corrupção, identidade ou commit incerto
falham fechados sem confirmar a nova obrigação. Não aumentam quotas.

Received persiste PUBLISH canônico antes da saída/ACK. O tópico é resolvido,
alias transitório e DUP são removidos; propriedades binárias, sequência de
User Properties/Subscription Identifiers e expiração recebida são preservadas.
Identidade semântica ignora somente expiração restante, que pode diminuir;
checksum do documento cobre todos os bytes. ID vivo com conteúdo divergente
é recusado, inclusive QoS1 Received cuja saída ainda não ocorreu.

Após saída/flush ou filtro, ReadyAck(QoS1)/AwaitPubRel(QoS2) é durável antes
de PUBACK/PUBREC. PUBREL torna PubCompPending durável antes de PUBCOMP.
Outgoing terminal, emitido após write/flush pelo event loop, permite limpeza;
erro de limpeza continua explícito. Fases terminais duráveis podem ser
coletadas ao admitir outra publicação, sem apagar Received/AwaitPubRel.
PUBREL conhecido retomado completa sem repetir saída; desconhecido recebe0x92
no opt-in. Após conclusão, ID reusado inicia publicação nova.

O fork rumqttc possui modo explícito para PUBCOMP manual e não usa bitset
transitório como prova de recuperação. Modo padrão continua autoACK e rejeita
PUBREL desconhecido; IncomingPubRel válido já autocompletado não encerra mais
a assinatura contínua. Canal opt-in32 e enqueue não bloqueante evitam aguardar
a própria fila; prazo absoluto da obrigação não é renovado por duplicatas.
Copyright e licença Apache-2.0 originais permanecem inalterados, com patch
descrito em XMQR-PATCHES.md. Projeto continua MIT, sem relicenciar terceiros.

## RED real preservado e revisões

ELF anterior isolado SHA256
415222bdc6855eaeba6130829225c82df0d740f7015159f4c22e82e93c3d1a8e:

- [Crash após PUBREC](../../docs/evidence/client-c129-inbound-red.json):
  SIGKILL, SP1 e PUBREL produziram EOF em vez de PUBCOMP; exit1,0.5474s.
- [Assinatura padrão contínua](../../docs/evidence/client-c129-default-red.json):
  encerrou depois do primeiro PUBCOMP, antes da segunda mensagem; exit1,0.3489s.
- Novo negativo unitário QoS1 Received/conteúdo divergente:0PASS/1FAIL,
  0.06s, antes da guarda mínima (prompt v1.9/v1.10).

Outros FAILs não ocultados, registrados antes de corrigir no prompt:
formatação inicial; build E0432 por bytes ser somente dev-dep (resolvido sem
mudar Cargo); fixture count inválida (CLI já recusava antes de CONNECT);
Clippy assert_is_empty/default_trait_access/too_many_lines; agregador WSS
exigindo marker inexistente apesar de harness exit0. Nenhum desses FAILs foi
promovido a PASS sem nova execução. Relatórios intermediários permanecem dist/.

## Gates efetivamente executados

Ubuntu26.04/WSL2 x86_64, Rust1.99.0. TMPDIR=/var/tmp, cargo locked/offline,
um job, cache privado já existente /var/tmp/xmqr-client-p92-native-e688c75;
não foi copiado nem limpo. ELF debug final:

109ec97326500b8edfe9c54f09318103f661866a3e16927a9b63e8d4f473d3e9

| Gate | Resultado real |
| --- | --- |
| cargo fmt --all -- --check | PASS |
| rustfmt --check dos dois arquivos do fork | PASS |
| cargo test --locked --offline integral |175PASS:2 bridge+104 client+1 unix+68 wire;0FAIL/0ignored |
| Novos unitários inbound incluídos no integral |8PASS: fases, identidade, quotas, atomicidade, permissões/lock/symlink, metadados e GC |
| Novos wire default incluídos no integral |2PASS: duas QoS2 contínuas; PUBREL desconhecido sem PUBCOMP |
| cargo clippy --locked --offline --all-targets -- -D warnings | PASS |
| cargo build --locked --offline --bin mqtt-client | PASS |
| python3 scripts/license_inventory.py --check | PASS; inventário declarado, não auditoria legal integral |
| Agregador Python, negativos de marker/exit |4PASS |
| [TCP C129 final](../../docs/evidence/client-c129-inbound.json) |18/18PASS,7.3372s |
| [Compatibilidade final](../../docs/evidence/client-c129-compatibility.json) |6/6PASS,11.7906s |
| Windows cargo check --locked --offline -j1 --bins --target-dir target/c129-windows | PASS,5m09s, Rust1.98.1/MSVC |

Windows usou diretório de build novo e próprio. Houve um warning preexistente
em mqtt-bridge.rs123, variável meta: arquivo idêntico à base e pertencente a
C128; não foi corrigido fora do ownership. Não se afirma execução de testes
Windows nem MSRV1.88 por esse check.

Os18 TCP cobrem SIGKILL após PUBREC/SP1/ID reusado, default contínuo,
SP0/corrupção/permissões/identidade/lock sem replay/ACK, stdout /dev/full em
QoS1/2, recuperação Received, conflito QoS1 vivo, deadline com quatro DUPs,
commit PUBREL falho sem PUBCOMP, desconhecido/repetido0x92, lote32,
count recusado sem rede, cancelamento e timeout total idle.
Compatibilidade executa C105/C107 outboxes, C108 manifesto, C110 ACK após
saída/filtros, C123 assinatura WSS e C114/C116/C119/C120/C122 WSS/mTLS/perfil/
outboxes e recuperação. Os harnesses originais não foram alterados: agregador
injeta somente o caminho explícito do ELF em seus módulos de teste.

## Reproduzir sem tocar target original

```text
export TMPDIR=/var/tmp CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR=/var/tmp/xmqr-client-p92-native-e688c75
cargo test --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo build --locked --offline --bin mqtt-client
python3 -B scripts/verify_subscription_inbound_journal_mqtt5.py --client "$CARGO_TARGET_DIR/debug/mqtt-client" --report dist/c129-inbound-new.json
python3 -B scripts/verify_c129_compatibility.py --client "$CARGO_TARGET_DIR/debug/mqtt-client" --report dist/c129-compatibility-new.json
python3 -B -m unittest discover -s scripts -p test_c129_harness.py -v
```

Use cache/diretório próprio disponível ao reproduzir; esses caminhos foram
do laboratório desta execução, não configuração do operador. Relatórios
exigem nome novo para não substituir FAIL anterior. JSONs anexados são
sanitizados, sem payload, segredo, certificado/chave ou log bruto.

## Limites e integração pendente

SIGKILL prova queda de processo, não perda de energia nem todas as falhas de
filesystem. A fronteira PubCompPending foi reaberta por teste unitário do
journal; TCP prova commit falho após PUBREL sem ACK, não promete que SIGKILL
atingiu cada instrução entre commit e envio. Não houve nova campanha física.
Janela stdout flush→commit pode duplicar saída após queda: stdout não é uma
transação com o consumidor. ACK confirma protocolo, não negócio ou persistência
do consumidor. QoS1 também admite duplicatas legítimas; QoS0 não ganha journal.

WSS preservou assinatura persistente QoS1 e recuperação das outboxes QoS1/2;
a nova queda do receptor QoS2 foi exercitada em TCP. Integração exata com C128,
versão final, release ELF, broker/admin, peers externos, gates globais6B/6C,
distribuição e publicação cabem ao coordenador. Nenhum resultado local deste
registro substitui a matriz global de migração, relógio, crash/restart, quotas,
Will Delay/expiração e interoperabilidade da fase6.

## Integração do marker, C129 v1.11

Sem novo runtime: prompt1606f7105b9f4d97a7a9547709c406054daff2c7 antes
do patch e552b7951ca4e7d9a5d31a35dbd81ce1f4eb701f no harness. A campanha
completa bem-sucedida agora emite C129_INBOUND_JOURNAL_PASS, além do JSON;
--case parcial não emite esse atestado. Compat mantém C129_COMPATIBILITY_PASS.

[Nova execução](../../docs/evidence/client-c129-inbound-marker.json):18/18PASS,
5.832s, mesmo ELF109ec97326500b8edfe9c54f09318103f661866a3e16927a9b63e8d4f473d3e9.
SHA do harness275489e3994f07cfa5b59a769c66c156ca2f3cd8aa40747148b3226683bf9213;
SHA do relatório1076ef0c0314a8801e2bcfc133e99a49eb6a62b228c2f7ee8a64acea7cf211af.
Marker específico observado após statusPASS. Provas negativas adicionais:
--case continuous no ELF final PASS,0.7113s, sem marker do conjunto; --case
recovery no ELF anterior FAIL/exit1,0.4719s, também sem marker. Relatórios
novos dist/c129-marker-subset.json e dist/c129-marker-negative.json preservados.
Oráculos, cenários, prazos e fontes Rust permaneceram congelados.

## Revisão independente C129 v1.12–v1.18 — correções e novo aceite local

Base do clone próprio5ba9cf6, runtime169c03a; o original C128 não foi tocado.
Promptv1.12 commitado localmentef139b5b antes dos testes/fixes. Três defeitos
foram separados da janela stdout→commit já declarada:

- Inbound órfão sem manifesto era tratado como primeira assinatura, recriando
  manifesto e conectando. Agora abertura/preparação recusam antes de limpar
  temporário/criar manifesto/rede; documentos e temporário ficam intactos.
- Hash da obrigação incluía SubscriptionIdentifiers mutáveis. OASIS §3.3.4-4
  e §3.8.3.1 permitem ordem/conjunto distintos em replay. Agora comparação
  ignora esses IDs; wire original/ordem/metadados continuam no checksum e no
  documento. Topic/payload/QoS/UserProperties e demais metadados não relaxados.
- Fork PUBLISH contava duas vezes o ID da propriedade SubscriptionIdentifier,
  podendo perder propriedade posterior e deslocá-la ao payload. Patch mínimo
  em publish.rs soma somente bytes do VBI; notice/copyrights preservados.
  Parser SUBSCRIBE semelhante não foi modificado fora do escopo autorizado.

### Compatibilidade explícita do draft inbound

Somente subscription-inbound.json passa de versão1 a2 pelo algoritmo novo.
Outboxes C105/C107, manifesto C108 e versão de pacote continuam inalterados.
Leitura v1 exige checksum e identityv1 originais válidos; normaliza em memória
sem reescrever arquivo/wire. Próxima transição atômica grava2. Versão99 e
identityv1 adulterada com checksum do documento recalculado foram recusadas
nos unitários; não há reinterpretação silenciosa de corrupção.

### RED preservado antes das correções

ELF anterior real109ec97326500b8edfe9c54f09318103f661866a3e16927a9b63e8d4f473d3e9,
copiado para área própria congelada, nunca apresentado como candidato novo:

| Prova | Resultado |
| --- | --- |
| Unitários review com2IDs, antes do fix |4FAIL:openórfão/prepareórfão/identidade/revisãoformato; compile6.63s/test0.12s,exit1 |
| [Órfão TCP](../../docs/evidence/client-c129-review-orphan-red.json) |FAIL/exit1,1.4315s, CONNECT indevido |
| [Ordem TCP](../../docs/evidence/client-c129-review-order-red.json) |FAIL/exit1,1.3449s, EOF antes PUBREC no replaySP1 |
| [Multiplicidade TCP](../../docs/evidence/client-c129-review-multiplicity-red.json) |FAIL/exit1,1.3985s, mesmo estágio |
| Parser write/read público antes do patch |2FAIL:ContentType posterior ausente;3IDs pequenos e VBI1/2/3/4bytes,16.16scompile/0.00stest |

Primeiros vetores3IDs falharam antes da identidade, por erro do parser;
dist/c129-review-order-red.json e multiplicity-red.json preservam essa prova,
mas somente as repetições red2 acima demonstram o conflito semântico isolado.
Primeiro REDunitário4FAIL/11.08scompile/0.08stest também preservado no histórico
do prompt. O comando do parser incluiu leitura posterior de noticeexit0;
esse status agregado não foi confundido com o cargoRED, que mostrou2FAIL.

Primeiro Clippy recusou dois estilos dos testes (literal/slice), corrigidos
sobv1.17 sem allow/oráculo novo. AST teve uma chamadaWSL/Service/0x8007274c
que não iniciou; repetição AST2PASS. Inventário falhou por cargo fora doPATH
no subprocesso WSL direto; repetido com PATH local explícito/CARGO_NET_OFFLINE
sem instalação/configuração global, PASS. Windows mantém warning preexistente
meta/mqtt-bridge.rs123 alheio; arquivo não alterado.

### GREEN final das fontes corrigidas

ELFdebug próprio congelado
`../p227-binaries/mqtt-client-c129-review-final-0.36.0` (relativo ao clone), SHA:
`52d4c8119f5da61ffa97b6147e0b51a5f6f8c868283efcd162d1fd1893c7a8ab`.
Nenhuma versão de distribuição/tag foi criada. Fonte runtime mudou somente
outbox.rs e uma linha do parserfork; v5_session.rs permaneceu intacto nesta
revisão. Todos os comandos Rust locked/offline-j1 usam cache privado anterior.

| Gate final | Resultado real |
| --- | --- |
| cargo test --locked --offline -j1 integral |181PASS=2bridge+110client+1websocket+68wire,0FAIL/0ignored; compile3.91s,client3.37s,wire14.59s |
| Direcionados review, já contidos nos181 |6PASS,17.57scompile/0.09stest; não somar novamente |
| cargo clippy --locked --offline -j1 --all-targets -- -D warnings |PASS,2.67s |
| cargo fmt --all -- --check e rustfmt--check forkpublish |PASS |
| cargo build --locked --offline -j1 --bin mqtt-client |PASS,3.77s |
| license_inventory.py --check, PATH explícito/offline |PASS; não é auditoria legal integral |
| Contratos Python do agregador |4PASS, sobrepostos à finalidade de classificação, não somados a Rust |
| Windows cargo check --locked --offline-j1 --bins --target-dir target/c129-windows |PASS,1.93s, warning alheio acima; não teste runtimeWindows |
| [TCP integral atualizado](../../docs/evidence/client-c129-review-inbound.json) |21/21PASS,8.6118s, C129_INBOUND_JOURNAL_PASS |
| [Draftv1 real→v2](../../docs/evidence/client-c129-review-v1-migration.json) |PASS,0.3019s, cleanupPASS, C129_V1_MIGRATION_PASS |
| [Compatibilidade](../../docs/evidence/client-c129-review-compatibility.json) |6/6PASS,10.3605s, C129_COMPATIBILITY_PASS |

Os21 TCP são18 anteriores maisórfão/ordem/multiplicidade. Sem ampliar recursos
nem reinterpretar testes antigos como nova cobertura. Compatibilidade repete
C105/C107/C108/C110/C123 e o conjunto WSS C114/C116/C119/C120/C122.

Migração real usa `verify_c129_v1_migration.py --legacy-client` apontando ao
ELF109ec973 e `--client` ao52d4c811. Journalv1 foi produzido pelo binário
anterior, não pela edição de um campo version:crash apósPUBREC, SP1 e replay
com IDs removidos não exibiram nova saída nem alteraram bytesv1; PUBREL/
PUBCOMP gravou2 preservando wire original enquanto presente. Unitário separado
cobre documento reconstruído com algoritmo antigo, não confundido com o real.

Hashes dos harnesses finais:

- TCP21:`8a5a982387bf8c1bcfe252cf66e61ef4bd2948937e31e7d8e9c52827d729ebda`.
- Migração:`dec6578665808b25348199bfc0a8c70b58f0ab026357e780fdb4f0fd92d2143a`.
- Compat:`5be2d5b8599ca431ff74f2e92964756f72a2fef5c5d73ea50dd28c554fe5f296`.

Reportoriginais exclusivos dist/c129-review-*.json e cópias sanitizadas em
docs/evidence/client-c129-review-*.json. SHA e tempos são deexecuções locais,
sem alegação de produção/SLA/powerloss/novoTLS físico. Ainda cabe ao coordenador
integrar C129 com brokerE/v16/admin e peers externos, validar snapshot/versões
finais e publicar. Sem push/tag/serviço do operador nesta frente.
