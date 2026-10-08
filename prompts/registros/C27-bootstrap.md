# Registro C27 - bootstrap independente e fechamento Unix C26

2026-10-08. Prompt clients/27-bootstrap-independente.md revisao1.0 gravado antes
DOS NOVOS FONTES. Rev1.1 antes dos gatesUnix coordenados, rev1.2 antes do harness
integracao/TLS, rev1.3 antes do ajusteClippy1.99/fixture0600. C28 apenas PLANNED.
Skill xmqr-client-development; AGENTS/README/ORIGIN/historico lidos.

Entrada: HEAD26c10191c9ec0b6d3181488794584bfbf24ed756, originlicodevone/xmqr-client,
manifest0.7.0local herdado/publish=false; tagv0.1.0/f8999e6 preservada.
Working treeC26 com alteracoes autorizadas mantido; sem limpar/reverter/commit.
Nenhuma alteracao de versao/tag/publicacao ou instalacao. Broker pausado pelo pai.

## Entrega/metodo

C27 e contrato consolidado da CLI e funcionalidades reais ateC26: Rust2024,
MSRV declarado1.88, rumqttc0.25.1/manifest/lock/features, TLS/SAN/mTLS/senhaoculta,
Linux0600/owner, quotas1024/4096/PEM1MiB, QoS/retain/sessoes/Will, reconexao0..10,
JSONL, timeout/cancelamento e limitacao recepcaoQoS2 interrompida.

validation/c27-bootstrap nao existia e foi criado vazio APOS persistir prompt.
Cinco modulos Rust novos (args,security,format,engine,main) foram redigidos nesta
etapa; nenhum srcprod ou fonte de implementacao broker copiado. Reutilizados
EXPLICITAMENTE Cargo.toml/Cargo.lock/LICENCA e fixturesC26tests/wire.rs como
oraculo de contrato. inputs.json guarda hashes iniciais. approved_asset_changes
registra apenas3asserts equivalente paraClippy1.99; checker/provenance.json
conferem insumos e hashes diferentes dos novos fontes.

Mesmo autor ja conheciaC26: NAO e ensaiocego nem prova independente de autoria
por hash. Manifest e lock sao dados declarados do conjunto, nao prompts que
fixam por si so todas transitivas. CacheRust e toolchains existentes usados.
HelperPython de integracao usa --fixtures explicito apontando ao harness do
broker; apenas teste adicional, nao dependencia do build/runtime do cliente.

Diferencas deliberadas: arquitetura/nomes internos novos; help e mensagens de
validacao/precedencia de erros nao sao textualmente identicos. Comparacao e por
contratos/exit/flags/JSON/wire nos cenarios testados, NAO equivalencia bitabit
nem garantia de todos comportamentos futuros. Crate de evidencias nao substitui
src do cliente de producao; version0.7.0 nela e dado, nao produto/release novo.

## PASS

| Binario atual | Windows Rust1.98.1 | Linux Ubuntu26 Rust1.99.0 | Integracao isoladaLinux |
| --- | --- | --- | --- |
| Cliente C26 em src/ | 28testes (19unit+9wire) | 30testes (21unit+9wire) | 9Will/retained +4C27contracts =13 |
| Reconstrucao C27 | 20testes (11unit+9wire) | 20testes (11unit+9wire) | 9Will/retained +4C27contracts =13 |

Em AMBOS crates/plataformas: cargo fmt --all -- --check; cargo test --locked
--offline; cargo clippy --locked --offline --all-targets -- -D warnings;
cargo build --locked --offline: exit0 finais. Windows aviso de linker criacao
.lib/.exp informativo; Clippy -Dwarnings PASS. Nenhum gate enfraquecido.
RootWindows cargo1.98.1; Ubuntu-26.04 cargo/rustc1.99.0, somente stable instalado.

Targets separados: root target/c27-linux-production; reconstrucao target/linux;
Windows target/debug de cada crate. Nao compartilhar build do broker.
Inventario Linux CARGO_NET_OFFLINE=true python3 -B scripts/license_inventory.py
--check: Project MIT declaration and dependency license inventory match.
Manifest/lock originais preservados e iguais aos assets declaradosC27.

Nove fixturesC26wire reutilizadas declaradamente validam pub/subQoS0/1/2,
EOF/auth/peerpayload sem leaks/retries indevidos, count antesSUBACK,
ID/sessoes estáveis, tentativas finitas, pub sem replay e QoS2interrompido.
UnitariosC27novos incluem todosbytes0..255/controlesJSON, CLInegative/limites,
precedencia flagsPEM/env, segredo sem echo, PEMbounded, cancelamento/livefila,
Linux0600 e arquivoWindows rejeitado. Quantidade de unitarios difere do original.

## Integracao real/coordenacao

Autorizacao pai: brokerP43 pausado, binario congelado somente testeisolado.
Binario usado: /mnt/c/Users/licod/Documents/Codex/2026-10-07/task/p43-broker-bin.
Script do broker verify_last_will.py executado com --broker e --client explicitos
para C26atual e C27, NAO usar resultado baseline26c1019 como evidenciaC26.
Portas127.0.0.1efemeras e state testowned TemporaryDirectory, sem estadoreal.
TMPDIR root target/c27-integration-tmp para state; nenhum arquivo broker editado.
RegressoesWill incluem crash/keepalive/takeover/restart/UTF8tópico1024,
retainedreplace/delete/wildcard e modo clean_sessiontrue/false. NovePASSporbinario.

validation/C27-integration.py: --broker BIN --client BIN --fixtures
/mnt/d/projects/my-project/xmqr/scripts/verify_interop.py. Quatrotestes porbinario:
1.JSONbinario/retain/tópico1024UTF8/QoS0,1,2 e conclusaopub correta;
2.restart do brokerproprio com subpersistente/clean, ID e count preservados;
3.SIGINTlive suprimeWill/linklive e SIGINTbackoff sai0limitado, sempub;
4.peerTLS Pythonssl/OpenSSL existente, mTLSvalido, CAerrada e SANerrado:
semCONNECT nosnegativos, semretryTLS e semsecret JSON/logs.

TLSpeer usa certificado/clientidentity gerados somente em tempdirLinux/tmp
700 e senha aleatoriafixture0600; nenhum PEM/key/password salvo noGit/logs.
NAO comprova secureintegrationcombroker real, somente TLSpeer controlado.
C26 TLS repetido seletivamente depois de corrigir filesystemfixture; 3casosPASS.
Todos processos e cleanup testowned; nenhum servico compartilhado alterado.

## FAIL intermediarios resolvidos

- ReconstrucaoWindows Clippy: boolsstruct/matchwildcard/letelse/Stringnew;
  ajustada nova implementacao antes do gatefinal, sem relaxar lints.
- ClippyLinux1.99 assert_is_empty apontou3asserts no root e copiafixture;
  trocados por assert_eqempty somente. Criteria/testscope iguais, finaisPASS.
- TLSprimeirafixture criou segredo emDrvFS/mnt/d: permissoes0600 nao refletidas,
  cliente RECUSOU corretamente antes deCONNECT. Corrigida fixturepara/tmpnativo,
  checagens seguras do cliente intactas. Nenhuma falhaTLSreal mascarada.
- WSLsem escalacao nao via distros; abreviacao WSL26 nao era nome real. Lista
  autorizada confirmou Ubuntu-26.04, usadosem instalar/reconfigurar.
Nenhuma falha final de gates ou dos cenarios executados pendente.

## BLOCKED / limites nao comprovados

- MSRV1.88 exato: toolchainnaoinstalado Windows/Linux, proibida instalacao.
  Metadata nao declara dependencias com MSRV>1.88, mas nao substitui esse build.
- Mosquitto externo, brokerTLS/mTLSsecure completo, ACL/auth validaproducao,
  testes de carga, cenariosnormativosnaocobertos: NOT_RUN, sem alegar aceite.
- Ctrl+C realWindows nao injetado; SIGINTLinux e componentesunitariosPASS.
- RecepcaoQoS2interrompida continuafalhaexplicita, sem reconectar fase parcial;
  rumqttc0.25.1 clean perde incomingstate. Nao prometer exactlyonce nessa queda.
- Reconstrucaonaocega/equivalenciabitsnaoatestada; insumos declarados necessarios.
- C28 apenasplanejamento. Arquivo/stdinbinario/perfis/multitopicos/timestamps
  NAOimplementados. Proxima execucao deve reconferirGit e resolvercriteriosC28.

## Preservacao/inventario

check_provenance PASS: seis fontesprodSHA256 iguais a entradaC27; novosfonte
hashes diferentes. Unica mudanca codigo-testroot nestaetapa:3assertsempty,
explicitadas com hashbefore/after. Nenhum brokerfonte editado, nenhum secretregistrado.
Arquivosatuais incluem Rust(runtime), Python(3scriptsapoio:licenseinventory,
integracaoC27 e provenance), TOML/JSON(config/metadados), Markdown(docs).
Nao existe clientePythonalternativo nem scriptShell/PowerShell/JS no projeto.
PowerShell/bash foram executores de comandos; scripts temporariosNode de edicao
nao sao fontes do aplicativo e nao foram incluidos como dependencias.

CheckpointC27/C26 concluido nas plataformas/cenarios acima, sem release.
Usuario decideversao/publicacao antes de continuarC28 ou outros recursos.

## Mudanca externa detectada na verificacao final

HEAD atual6d25b335e28ec9df1346c5a0270ef49747cf8937. Durante a execucao surgiram
commits externos incluindo dea9b72,3881a89,6d25b33 com os artefatosC27.
Esta tarefa NAO executou gitcommit/push/tag/release; commits externos preservados,
sem reset/rebase/rewrite. Manifest continua0.7.0/publishfalse e taglocalv0.1.0
inalterada. Registro de entrada26c1019 e historico, nao HEADfinal.
Nenhum comando travado ao checkpoint: todos gates/integracoes finalizaram.
