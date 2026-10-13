# Registro C46 — candidato de distribuição 0.9.0

2026-10-13. Prompt C46 v1/v1.1 e revisões P92 preparados antes das mudanças
de versão, embalagem e correções. A skill local xmqr-client-development
orientou a preservação do contrato e os gates. Publicação coordenada pelo
agente raiz; este registro não declara que a fase 3 ou a release foi encerrada.

## Fonte exata e resultado nativo

Fonte de distribuição: **e688c75a3779803e8f1868792dcc5e7a36f2dd6e**, versão
**0.9.0**, clone detached e clean antes/depois dos gates. O primeiro clone
6254d066 foi atualizado para e688 após o suplemento original do rumqttc.
O main posteriormente avançou, externamente a esta execução, para 03d2447
(documentação/artefato antigo); isso não troca a fonte identificada do pacote.

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

## Artefatos candidatos

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
erro do build; o ciclo de instalação/upgrade/rollback e execução do pacote
permanece gate separado do coordenador. Não inferir suporte Ubuntu24/arm64.

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
   concomitante foi invalidado e deve ser repetido; não é PASS de robustez.
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

## Pendências coordenadas

P91 nos dois brokers e ensaios ampliados, B06/B07 final, ciclo dos pacotes,
Docker/digests/manifest completo, tag imutável v0.9.0, GitHub prerelease e
Docker Hub serão registrados pelo coordenador **após** os gates respectivos.
Não foram feitos commit/push/tag/publicação por este agente. Não iniciar
fase4 ou MQTT5 para concluir esta entrega. Sem nova homologação Windows
nativo, MSRV1.88 exato, arm64, produção ou conformidade MQTT integral.
