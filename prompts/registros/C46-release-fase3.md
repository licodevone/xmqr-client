# Registro C46 — aceite e distribuição experimental 0.9.0

2026-10-13. Prompt C46 v1/v1.1 e revisões P92 preparados antes das mudanças
de versão, embalagem e correções. A skill local xmqr-client-development
orientou a preservação do contrato e os gates. C46/P92 v1.7 encerram o aceite
técnico da fase 3 no recorte finito validado. Publicação coordenada pelo
agente raiz concluída e verificada conforme P92 v1.9, na seção própria abaixo.
Aceite técnico isolado não foi usado como prova de publicação remota.

## Fonte exata e resultado nativo

Fonte de distribuição: **e688c75a3779803e8f1868792dcc5e7a36f2dd6e**, versão
**0.9.0**, clone detached e clean antes/depois dos gates. O primeiro clone
6254d066 foi atualizado para e688 após o suplemento original do rumqttc.
O main posteriormente avançou, externamente a esta execução, para 03d2447
(documentação/artefato antigo) e bf81396 (registro C46). Esses checkpoints
documentais não trocam a fonte identificada do pacote. Neste fechamento,
src/Cargo/Dockerfile foram conferidos iguais ao checkpoint e688.

Ambiente: Ubuntu 26.04/WSL, Rust 1.99.0, build nativo amd64; execução direta
WSL, PATH explícito do Rust, TMPDIR e CARGO_TARGET_DIR privados em **/var/tmp
ext4**, concorrência de build -j1. Não foi reaproveitado o cache defeituoso.
O driver auxiliar gerado ficou somente em clone/dist, fora dos fontes Git.

| Gate | Resultado real |
| --- | --- |
| fmt --all -- --check | PASS, 0,426s |
| cargo test --locked | PASS, 63 testes: 34 unitários + 29 wire; 0 falhas/ignorados; 163,583s com compilação fria |
| Clippy locked, all-targets, -D warnings | PASS, 83,809s |
| build locked --release --bins | PASS, 178,792s |
| Inventário de licenças --check | PASS |
| Contratos de distribuição | PASS, 7 testes no Linux; os mesmos 7 também passaram no Windows/Python3.13 |
| Coleta dos textos originais | PASS, 81 pacotes / 158 textos, superset do Cargo.lock |
| Geração .deb Ubuntu 26.04 amd64 | PASS; metadados Package/Version/Architecture/Depends consultados depois |
| Git após os gates | Clean, HEAD e688c75 |

Duração total do driver: **434,906s**. Esses tempos não são benchmark nem SLA.
Logs/JSON reais em `.artifacts/phase3-p92-20261013/xmqr-client/dist/`, no
agrupador local, não no Git: c46-native-gates.log e c46-native-final.json.

## Artefatos aceitos

- Executável `dist/native/mqtt-client`: 6.116.272 bytes; versão retornada
  `mqtt-client 0.9.0`; SHA-256
  `a7a89dba2245f56d4161d52a9767441257edca04be8042edcabf358e4bb05f58`.
  É igual ao executável previamente ensaiado em C45 (incluindo idle90s real
  e SIGINT/SIGTERM); por esse resultado não foi repetido o probe90s no clone.
  Igualdade observada neste build não é promessa geral de builds reproduzíveis.
- `dist/native/xmqr-client_0.9.0-1_amd64.deb`: 1.702.612 bytes; SHA-256
  `04eb2830dc67fa086f327555f58d027f8f4e4c1654f72ddc000589ab90c05396`.
  Checksum acompanhante gerado pelo builder. Package xmqr-client, Version
  0.9.0-1, Architecture amd64. Dependências derivadas do ELF: libc6>=2.38,
  libgcc-s1>=4.2, adduser e init-system-helpers.

dpkg-shlibdeps emitiu advertência de diversion do loader por usr-merge, sem
erro do build. O ciclo de instalação/upgrade/rollback e execução do pacote
foi validado separadamente pelo coordenador, conforme fechamento abaixo;
não foi inferido desse aviso. Não inferir suporte Ubuntu24/arm64.

O coordenador também confirmou QA read-only dos textos no payload desse
`.deb`: 81 pacotes / 158 textos, bytes estrangeiros estritos e MIT/copyright
canônico do clone. Uma chamada inicial do wrapper dele usou caminho de
manifest inexistente (FileNotFoundError); a chamada corrigida passou. Não
foi falha do artefato nem correção de fonte neste registro.

## Licenças e identidade de bytes

O código próprio continua MIT, sem relicenciamento das dependências.
Inventário/coletor incluem dependências dev e de outras plataformas, não
afirmam composição completa do ELF. A coleta verifica SHA do original e da
cópia; os suplementos valuable/rumqttc têm revisão/proveniência pinadas.

MIT canônico Git/clone: SHA-256
`84381e555814f7125a9db5bce4f79b786cfb795b05eaddc2ef31a3a414842fc9`.
O checkout original Windows em CRLF tem SHA diferente
`acfd4bb82db195b2ba0a8a1ea6c5048d2e66c05a0fd0bd843e08d8a46d34b465`;
QA compara o pacote com o clone/Git canônico, sem modificar o texto.

Suplementos originais preservados byte a byte:

- valuable0.1.1: `ed60d479b8fd1f64e9cbc3de449a16a53ac1b3d1b6aeb9bf9d190a8e93061b44`.
- rumqttc0.25.1, Apache2.0, revisão CargoVCS
  `f1e9e8d558783f942993046679cdf3c8c3a3d36b`:
  `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`.

## Falhas anteriores preservadas — não convertidas em PASS

1. Tentativa de copiar somente cache por loop sofreu expansão de variáveis
   na passagem Windows→WSL: os subpaths viraram target//, copiando mais cache
   do que o previsto para diretório próprio em /tmp. Não alterou o original.
2. /tmp era tmpfs de 3,9GiB e ficou sem espaço; inicialização da dependência
   libc no primeiro gate nativo falhou com ENOSPC. O ensaio prolongado P91
   concomitante foi invalidado e exigiu repetição; não é PASS de robustez.
   O novo ensaio aprovado abaixo não modifica esse resultado histórico.
3. Várias invocações/kill escopado por WSL falharam com Service0x8007274c.
   Foram encerrados **apenas** os launchers Windows do job próprio,
   PID25596/42664, identificados por commandline, e a sessão83201 terminou
   com exit1. Isso não foi restart/shutdown da VM ou parada de serviço real.
4. Posteriormente o coordenador observou /tmp vazio, cache yZk7G1 ausente e
   PIDs Linux8685/8699 inexistentes. Seu helper de recovery recusou mover,
   porque origem/destino não existiam. **Não afirmar que limpamos/movemos o
   cache, nem inferir a causa dessa mudança de ambiente.** Nenhum move/delete
   foi executado por este agente. Relatórios anteriores foram preservados.
5. Após P92 rev1.3, o gate foi realmente repetido no ext4 /var/tmp e passou
   conforme tabela; a recuperação não apaga os FAIL/limitações anteriores.

Histórico factual local: clone/dist/c46-native-progress.json. Nada desse cache
amplo entrou em pacote/imagem ou foi publicado. Arquivos locais antigos0.7.1
não foram removidos, alterados ou republicados por esta execução.

## Fechamento técnico coordenado — C46/P92 v1.7

Resultados executados pelos responsáveis dos gates e reconciliados neste
registro; esta atualização documental não é uma nova execução funcional.
Os JSONs finais e o QA foram lidos, sem reescrever os relatórios históricos.

| Gate final | Resultado observado |
| --- | --- |
| C07 / client | 63 testes e oito gates no clone clean; ELF nativo igual ao C45, idle 90s real e cancelamento já validados |
| P91 / interop independente | PASS 87/87 (59 C43 +28 extensões), 346,063s; XMQR/Mosquitto 2.0.22 em ambos os brokers/sentidos, mTLS/ACL/reconexão/crash/idle; cleanup 202 containers/2 volumes PASS |
| P91 / pressão e recuperação do broker | PASS 6/6, TCP/mTLS paralelos em três repetições >=600s, 1808,946s de parede; 40512 publicações de 1024 bytes/162048 entregas, 90 SIGKILL/30 SIGTERM |
| Broker final / B06+B07+B08 | 54 casos de armazenamento, 18 processos P90, 19 wire e 12 TLS PASS; correções P93/P95 vinculadas ao ELF final |
| Compatibilidade do admin independente | PASS 18/18 contra broker final; versão 0.1.0 e pin/distribuição anteriores mantidos |
| QA de artefatos | PASS 5/5 payloads (dois .deb, três imagens) e 14 negativos; no client 81 pacotes/158 textos, bytes de terceiros estritos e MIT/copyright preservados |
| Ciclo Ubuntu26 privado | Install 0.11.5/0.8.1, upgrade 0.11.6/0.9.0, rollback/remove/purge PASS; os três ELFs instalados executaram --version real, sem autoenable e com fixtures/configurações preservadas |

Fonte broker de distribuição `abbe38ca6c041f7328e85054506a4eed22c74c41`;
ELF nativo broker
`e57b1e2d26de50d09bb6bdadb74e4c0531dcac15399ece4369f4e0a0824e2987`.
Fonte client continua **e688c75a3779803e8f1868792dcc5e7a36f2dd6e** e o
`.deb` continua SHA-256
**04eb2830dc67fa086f327555f58d027f8f4e4c1654f72ddc000589ab90c05396**,
sem rebuild, alteração de versão ou substituição do artefato por estes docs.

Interop usou imagem client ID
`sha256:e1c56183bd98fbe33c9fade58f81d61b2f60eb960c3704747528ec7f471a11ed`,
revision e688; broker ID
`sha256:576de5ddc2dc2f74cbb2f243126d8c36668784f110ed0da03a05df8435352f97`,
revision abbe. IDs locais de imagem não são digests remotos do Docker Hub.

O soak conserva HEAD 3a5e29d/dirty=true do início; as 26 entradas src/Cargo
ficaram estáveis e o ELF executado coincide com o rebuild clean abbe.
Não apresentar esse ensaio original como checkout clean. Seus 1698 probes
incluem quatro ready0 transitórios, recuperação máxima 1,659s dentro do
prazo quiescente 15s. Isso não é SLA instantâneo de readiness/throughput.
As causas exatas dos dois FAIL intermediários de readiness externa não foram
recuperadas; Will 6/contratos 21 e a matriz completa final reforçam a evidência
sem promover os FAIL antigos por simples retry ou ampliar -W8.

Relatórios finais no agrupador local, fora do Git:
`p91-soak-candidate-6.json` SHA
`6bdc2c71c276723a0c4ad782643752aad243816e5d8283a386d63678951826c1`;
`p91-interop-candidate-4.json` SHA
`809719cd60c24e8a97c53506b813719125470c3b1494275e164c06e3aaabddaf`.
QA conjunto: broker/dist/p92-notices-final.json SHA
`60b392dec0593f8e8994573a5d25b9f8dfc64ff624e10ff8860efa4c1d32d0ae`.
Contexto público da coordenação:
[aceite dos três sistemas](https://github.com/licodevone/xmqr/blob/main/docs/phase3-acceptance.md)
e [registro P92](https://github.com/licodevone/xmqr/blob/main/prompts/registros/P92-aceitacao-distribuicao-fase3.md).

## Publicação coordenada verificada — P92 v1.9

- Tag anotada **v0.9.0** aponta para a fonte exata
  `e688c75a3779803e8f1868792dcc5e7a36f2dd6e`; `git ls-remote` confirmou
  esse destino peeled. Não foi criada ou movida por esta atualização documental.
- [GitHub prerelease v0.9.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.9.0)
  publicada em **2026-10-13T22:23:52Z**. O coordenador conferiu nome, tamanho
  e SHA dos **18 arquivos anexados** por API antes de retirar o draft.
  A página publicada foi consultada nesta revisão; arquivos de fonte
  automáticos do GitHub não integram essa contagem de anexos do projeto.
- Docker Hub **lepinheiro100/xmqr-client:0.9.0**, digest remoto
  `sha256:e1c56183bd98fbe33c9fade58f81d61b2f60eb960c3704747528ec7f471a11ed`.
  O coordenador confirmou inspect remoto, pull/run por digest, `--version`
  e revision e688. O fato de coincidir com o ID de imagem registrado acima
  foi verificado, não presumido como regra geral do Docker.
- Downloads reais de `.deb`, seu sidecar, manifest e seu sidecar: quatro
  assets por release, **bytes idênticos aos locais**, exit0. O client `.deb`
  conserva SHA `04eb2830dc67fa086f327555f58d027f8f4e4c1654f72ddc000589ab90c05396`.
- A release coordenada do broker v0.11.6 também foi publicada, com 42 anexos
  e tag abbe; o admin independente permanece 0.1.0 sem repin/novo runtime.

O `.deb` é nativo **Ubuntu26.04 amd64**. Os exports `.tar.gz` são imagens
Docker **Bookworm Linux/amd64**, destinadas a `docker load`; não são pacotes
de executáveis nativos. Latest, tags/releases anteriores, assets originais e
os tar/checksum 0.7.1 do mantenedor foram preservados. Esta tarefa alterou
somente CHANGELOG, COBERTURA e este registro; README/guias ficam com a
coordenação, sem reconstruir ou substituir artefatos por commits documentais.

P92 v1.8 corrigiu a transcrição anterior do SHA da matriz (um 'd' extra,
65 caracteres). Este registro agora usa o SHA real de 64 caracteres,
conferido nos bytes do JSON, sem editar a evidência original ou alterar o gate.

## Limites do aceite e da distribuição

MQTT 3.1.1 continua o protocolo entregue. MQTT 5 é planejamento da fase 6,
não iniciado; fases 4/5 não foram implementadas neste fechamento. Sem nova
homologação Windows nativo, MSRV1.88 exato, arm64, ESP32 físico, produção,
SLA, conformidade integral, estabilidade de dias ou perda física de energia.
