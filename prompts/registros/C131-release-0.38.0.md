# C131 — distribuição experimental client0.38.0

Estado inicial2026-10-18:f3c71620, clone próprio e branchrelease/0.38.0.
Promptv1.1 commitado12a1d83 antes de manifests/CI/docs. GitHubtag/release e
DockerHubtag038 consultados e livres. OriginalC128 preservado; não é release
da bridge nem aceite global da fase6. Validação/publicação em andamento;
nenhum gate pendente é consideradoPASS.

## Publicação e fonte imutável

[Prerelease v0.38.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.38.0)
publicada em2026-10-18 com seis anexos. Tag anotada
`beac6e03ad7cae89efefcea9b8e044bd074c3be4` aponta para
`fca6fc82117b7ee2b770dc2e4ce673b53764b211` e foi conferida no remoto.
[CI exato](https://github.com/licodevone/xmqr-client/actions/runs/39138244703):
Rust, Ubuntu-package e bridge-integration PASS. A bridge permanece fonte de
desenvolvimento e regressão, não binário/unit no pacote/imagem038. Não houve
merge no originalC128 nem movimento de latest/tags antigas. Origin do clone
continua local; pushes foram explícitos para licodevone/xmqr-client no GitHub.

## Falhas preservadas e correções de harness

- CI512f85e/run39137723808 FAIL/exit2:5ba9cf6 não era árvore pública
  alcançável. C131v1.2 revisou antes do ajuste;235f20d é a fonte pública
  equivalente, diff src/vendor/tests vazio. Não foi falha de runtime.
- CIe5ef844/run39137783610 FAIL: OSError no cleanup, após campos MQTT e
  migração positivos. [Relatório original](../../docs/evidence/client-c131-ci-migration-fail.json).
  Promptv1.3/c87cbe2 antes de RED/fix. Regressão determinística observou filho
  vivo quando TemporaryDirectory começava a remover o diretório, exit1.
  ExitStack agora encerra/aguarda filho antes do listener/diretório, inclusive
  no caminho de erro. GREEN local e CI2153ba9/run39138107860 PASS; repetido na
  fonte final fca6fc8. Nenhum ACK/byte/oráculo funcional foi relaxado.
- Primeiro instalador descartável FAIL/apt100 antes de instalar: capsnone
  negou setgroups/seteuid doAPT. Reprodução direta EPERM; promptv1.4 antes de
  usar capacidades padrão apenas no instalador descartável sem rede.
  RuntimeMQTT permaneceu UID10001/capsnone/read-only. Sem privileged.
- Primeiro build local recebeu VCS_REF incorreto digitado: não aceito nem
  publicado. Build final calculou git rev-parse HEAD e metadados foram
  conferidos. Cache de compilação reutilizado, não a alegação de proveniência.

Migração helper atual SHA256
`a2aa613408ec603a2888b2408c39f7e0a9f6b0b908f130e629d3c8ccf0c033c3`;
anterior dec65786 preservado nos relatórios antigos. TCP21 continua
`8a5a982387bf8c1bcfe252cf66e61ef4bd2948937e31e7d8e9c52827d729ebda`.

## Validações locais e CI

RustWSL1.99.0/Ubuntu26, cache privado, locked/offline/j1.181PASS
(2bridge+110client+1websocket+68wire),0FAIL/0ignored; compile25.42s,
client3.32s/wire14.58s. Seis testes review já estão nos181, não somar.
Fmt, rustfmt do fork, Clippy all-targets -Dwarnings5.66s e inventário de
licenças PASS. Build release110s PASS. CheckWindows1.98.1 --bins no cache
próprio PASS3.24s; warning preexistente meta/mqtt-bridge.rs123 preservado.
Check não homologa execução/journalWindows. Nenhuma dependência foi atualizada.

ELF nativo038 congelado em área de artefatos do agrupador:
`p227-binaries/mqtt-client-c131-0.38.0`, SHA256
`a5ccc20519fbe22661e230956d657379452bec69e1829dd18047d30556f85703`.
Runtime/Cargo/lock não mudaram de512f85e até a fonte final fca6fc8; mudanças
posteriores nessa faixa são prompt/CI/harness de limpeza. CI final executou
os gates na fonte exata, sem reutilizar ELF036 como038.

| Evidência | Resultado |
| --- | --- |
| [Native inbound](../../docs/evidence/client-c131-native-inbound.json) |21/21PASS,6.8481s |
| [Native compatibilidade](../../docs/evidence/client-c131-native-compatibility.json) |6gruposPASS,6.4589s |
| [Migração nativa real](../../docs/evidence/client-c131-native-v1-migration.json) |PASS,0.3278s, ELF antigo109ec973 real→038a5ccc205 |
| [Migração CI real](../../docs/evidence/client-c131-ci-v1-migration.json) |PASS,0.1617s; histórico235f20d compilado, não documento editado |
| Classificador de fixtures |4contratosPythonPASS; sem somar aos181Rust |
| Regressão de ordem de cleanup |RED1→GREEN1, C131_MIGRATION_CLEANUP_PASS |

Migração preservou bytesv1 na leitura/replay e não repetiu saída; PUBREL/
PUBCOMP levou documento a2. Identidade e checksumv1 antigos são validados
antes de normalizar em memória; sem migração de outboxes/manifesto v1.

## Pacote, notices e ciclo descartável

`xmqr-client_0.38.0-1_amd64.deb`,2419808bytes, SHA256
`a496e9e2e5887d7211a7e616b43a00e51c3d3afa63beb0f5d60f4c6dc492c90b`.
Build nativoUbuntu26amd64, version038-1, CLI038, dependências derivadas doELF:
libc6>=2.38/libgcc-s1/adduser/init-system-helpers. Avisos usr-is-merged de
dpkg-shlibdeps preservados; não houve dependência forçada. Build/instalação e
systemd-analyze verify do pacote também PASS no job CIUbuntu26 da fonte final.

Container Ubuntu26 próprio, sem rede/mounts de escrita do host: instalou036
publicado (SHA904c9731...), upgrade038 e units/no-autostart/no-bridge PASS.
O038 instalado produziu journalv2 REAL, crash apósPUBREC; rollback036 manteve
bytes/modos/configuração/credencial sem executar o antigo sobre o journal.
Restaurar038 retomou SP1/PUBREL→PUBCOMP sem nova saída. Remove/purge preservaram
journal, credencial0600 e conta; C131_PACKAGE_LIFECYCLE_PASS,3.1151s.
Programa reprodutível em dist/c131-package-cycle.py do clone isolado; não
alterou WSL instalado, serviços ou dados do operador.

[Conferência de notices](../../docs/evidence/client-c131-notices.json):
MIT própria e170pacotes/315textos originais/1644483bytes PASS no deb e imagem;
terceiros comparados byte a byte, sem normalizar licenças. A coleção é superset
incluindo dev/outras plataformas, não auditoria legal completa nem relicenciamento.

## Docker Hub e provas por digest

`lepinheiro100/xmqr-client:0.38.0` publicado, manifest/pull confirmados:
`sha256:b025a82ab4172f82223e8b01b125570996f374a27ee524a04cc62eac81a148f9`.
Childlinux/amd64 `sha256:727ba6e35466c185b296ef9b89bd4cebb64d1c92c2628a3e979a80a28ebe5f32`,
config `sha256:0fdc1d322aaf28c9b2635998e6d0ba037e479e5da166e298c7881772c9a40567`.
Labels038/fca6fc8, UID/GID10001:10001, entrypoint/versão/sinal PASS.
ELF da imagem SHA256 `fe2ece0319c6926ee6a8091463b3b1971637be1de1d066d713b940892c71b377`:
buildBookworm distinto do ELFUbuntu, sem alegação de identidade de bits.

[Smoke remoto](../../docs/evidence/client-c131-docker-remote.json): TCPmTLS
QoS1/pub/sub, restart/retained e WSSMQTT3.1.1/5/perfil, opt-in e negativos
CA/SAN/senha/ACL PASS por digest, rede/volumes próprios, sem portas host.
Par broker042 é o digest público f31da3e..., NÃO o candidatoE/v16.

[Journal21](../../docs/evidence/client-c131-docker-journal.json) PASS4.6309s
e [compat6](../../docs/evidence/client-c131-docker-compatibility.json) PASS6.3433s
no mesmo ELFfe2ece03. QA derivada FROM digest remoto apenas acrescentaPython
de teste, preservando o binário; UID10001/capsnone/RO/netnone, tmpfs privado.
Isso é fixture sintética do journal sobre runtime derivado, distinto do smoke
de pub/sub na imagem distribuída sem Python. Nenhum fixture/segredo foi
incluído na imagem final; volumes/containers de teste foram removidos.

## Anexos remotos e limites

[Verificação remota](../../docs/evidence/client-c131-remote-assets.json): seis
anexos baixados para diretório novo, tamanhos/SHA256 iguais aos locais,
sidecars e manifesto conferidos; REMOTE_ASSET_HASH_PASS6.
ExportDocker34563773bytes SHA2801154cc8f1c6d75b93bc66e14b1b260794f5ab9f7dc3a04a497600efd34a7d;
manifest1034bytes SHA8121861c5d45ebf2463dfbadda189db86c53a874aa803f83a886ff2e05882fcf.
Notices estão embutidos no deb/imagem. Repositório originalC128 e tags antigas
permaneceram intactos. Registro pós-tag não muda fonte/tag/publicação aceita.

Distribuição de client somente. Fase6 global/brokerE/v16/admin ainda em
validação; sem prontidão de produção/conformidade integral, powerloss/SLA,
exactly-once da aplicação ou suporteWindows ao journal. Stdout→commit
continua janela não atômica e ACKMQTT não comprova sucesso de negócio.
