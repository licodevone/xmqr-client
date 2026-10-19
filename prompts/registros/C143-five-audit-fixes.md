# C140–C143 — cinco falhas obrigatórias da auditoria MQTT 5

2026-10-18. Pedido explícito de correção, seguindo prompt→código→validação.
Execução sequencial em três fatias: capacidades; estado local; receptor QoS2.
Prompts C140/C141/C142/C143 foram criados antes da implementação. Base
`e03c5a46cfb33666896eb710892cf46e3c40ee38`; última release 0.41.0 em
`1d60d382782ec33ed876b8b43d1c87503df7ef86`. Preservada a saída não rastreada
`vendor/rumqttc-0.25.1/target/`. Broker e admin não foram alterados.

## RED e GREEN dos cinco achados

Servidor sintético TCP loopback/portas efêmeras, sem estado do operador.
P232 original foi executado sem modificar o harness nem a auditoria histórica.

| Obrigação | Binário publicado 0.41.0 | Candidato 0.42.0 |
| --- | --- | --- |
| MQTT-3.2.2-11, Maximum QoS | FAIL: enviou PUBLISH QoS1 sob máximo 0 | PASS: fecha sem PUBLISH, sem downgrade |
| MQTT-3.2.2-14, Retain Available | FAIL: enviou RETAIN1 sob anúncio 0 | PASS: fecha sem PUBLISH retained |
| MQTT-3.2.2-4, Session Present | FAIL: processo sem estado aceitou sessão anterior | PASS: encerra sem PUBLISH/SUBSCRIBE |
| MQTT-4.3.3-10, receptor QoS2 | FAIL: mesma transação exibida duas vezes | PASS: uma entrega e PUBREC repetido |
| MQTT-4.3.3-11, PUBREL repetido | FAIL: EOF sem novo PUBCOMP | PASS: PUBCOMP 0x92, mesma conexão utilizável |

RED `../.artifacts/c140-baseline-20261018.json`: 3 PASS/9 FAIL/0 BLOCKED,
SHA-256 `32f432748f7bae1e33261cf003e50724e79b7ba8bd5a94b39aa30349ed644605`.
ELF publicado SHA-256 `b9f2fa67c4c4d7263966097cd048627929b59d161b436d1b23e6005edac872f0`.

GREEN das cinco em `../.artifacts/c143-worktree-042-p232.json`: total 8
PASS/4 FAIL/0 BLOCKED, relatório SHA-256
`7d93d124e1cb031757643deeb95a671e3e48889e34fa2a3592a79738944859e0`.
ELF debug 0.42.0 SHA-256
`c80f1c5a372ce714ac5e359afbc65b2d127ec4e7408ec85c5a33b42a84b211a9`.
Esse relatório declara a base e03c5a4; trata-se do worktree modificado, não
de um build limpo daquele commit. A fonte exata final será identificada pela
CI abaixo, sem atribuir o novo ELF à release antiga.

Os relatórios completos, incluindo RED, recomendações ainda FAIL e regressões
GREEN, estão versionados em
[evidência JSON](../../docs/conformance/client-audit-fixes-evidence.json).

As quatro falhas restantes são recomendações de recepção: NUL em Topic Name,
NUL em User Property, CONNACK VBI não mínimo e flags reservadas PUBACK.
O FAIL global do harness P232 é esperado enquanto elas estiverem abertas;
não foi transformado artificialmente em PASS. Os dois controles positivos e
as cinco obrigações corrigidas passaram. Sem certificação ou conformidade global.

## PASS local

Ubuntu 26.04/WSL, Rust 1.99; cache nativo separado
`/var/tmp/xmqr-client-p92-native-e688c75`. Versão candidata 0.42.0/bridge 0.4.0.

- fmt --check, test --locked (193 testes: 7 bridge +112 client +5 receptor
  público +1 WebSocket +68 wire), Clippy --all-targets -D warnings, build
  --bins e inventário de licenças: PASS.
- Nova fixture independente `verify_mqtt5_audit_fixes.py`: 42/42 PASS.
  Cobre QoS0/1/2, retained permitido/negado, defaults, valores inválidos,
  repetição, pub/sub sem estado, primeiro manifesto, outbox QoS1/2 com
  recusas de capacidade/retained e hash de journal preservado, continuação
  PubRelReady sob capacidades reduzidas, duplicata/PID65535/reuso/PUBREL
  desconhecido e repetido. Relatório `dist/c143-wire-042-final.json`.
- Compatibilidade C129 contra o ELF 0.42.0: 6/6 módulos PASS,
  `dist/c143-compatibility-3.json`, SHA-256
  `08382e1e9bd9fefd61243de345ba5e7efce4ed38abd8a84df3f925b009f8aa2b`.
  Inclui outbox QoS1/2, manifesto, ACK após saída e WSS mTLS/recuperação.
- Journal inbound C129: 21/21 PASS antes do bump de versão, incluindo crash,
  retomada, corrupção/permissões/identidade/lock, falha de stdout, deadlines,
  32 publicações e PUBREL repetido/desconhecido. Repetição da candidata
  0.42.0 em `dist/c143-inbound-042.json` é registrada separadamente.
  Essa repetição terminou PASS 21/21, com o mesmo ELF 0.42.0 acima.
- Bridge MQTT5/QoS2: XMQR↔XMQR PASS incluindo dois checkpoints debug de crash;
  XMQR→Mosquitto e Mosquitto→XMQR PASS, TLS/mTLS, retained, ACL/CA/SAN/senha,
  reconexão. Esses ensaios locais precederam somente o bump de versão da
  bridge; o runtime C139 não foi reescrito. CI testa a versão exata novamente.
- Patch original Apache-2.0 preservado; nenhuma mudança MQTT3.1.1/TLS/SAN.

## FAIL/BLOCKED preservados e ajustes anteriores ao novo ensaio

- Compilação dos novos testes: Outgoing importado por módulo privado e
  ConnAckProperties sem Default. Corrigidos só os testes após revisões C142/C140.
- Clippy: run_inner >100 linhas, depois ref_option. Extraídos helpers de
  flags duráveis/alias e usado Option<&T>; sem supressão das regras.
- Primeira fixture ampliada: 35 PASS/5 FAIL por entradas inválidas do
  harness (repeat sem intervalo, diretório inexistente, manifesto com count).
  C143 v1.1 precedeu ajustes. Relatório `dist/c140-wire-1.json` preservado;
  segunda execução 40/40 PASS, depois cobertura ampliada para 42.
- Testes históricos esperavam duas violações antigas (Session Present=1 sem
  estado e PUBREL sem resposta). Contratos corrigidos após revisões C141/C142,
  preservando positivos/negativos. A nova saída JSONL exigia bytes [120], não
  busca pelo caractere x; FAIL do teste adaptado ficou registrado antes do ajuste.
- Compatibilidade C108 inicialmente falhou por diagnóstico novo; preservada
  a limpeza histórica apenas do manifesto novo nunca ativado. Existentes
  não são removidos. Registro `dist/c143-compatibility-1.json` permanece FAIL.
- WSL intermitente 0x8007274c ao iniciar processos: BLOCKED de infraestrutura,
  repetido sem reiniciar/parar serviços do operador. Inventário inicialmente
  BLOCKED por cargo ausente no PATH; passou com PATH explícito. Uma chamada
  WSL única executou os gates finais, evitando falsos resultados por comandos
  de validação que nem chegaram a iniciar.

## CI e distribuição — histórico da preparação e aceite final abaixo

Primeira CI [39183637497](https://github.com/licodevone/xmqr-client/actions/runs/39183637497),
fonte cf7d7c2: FAIL em bridge-integration antes dos ensaios de protocolo,
porque o harness ainda exigia `mqtt-bridge 0.3.0` após o bump para 0.4.0.
C143 v1.3 precedeu a correção da expectativa. O FAIL original é preservado;
release exige nova execução integral da fonte corrigida.

Build Docker local cf7d7c2: FAIL depois de compilar release, no assert final
do Dockerfile que ainda exigia bridge 0.3.0. C143 v1.4 precedeu o ajuste
para 0.4.0; não se publicou imagem nem tag. A segunda CI 39183873286 também
tem esse assert antigo e não pode servir como aceite integral da distribuição.

Preparada candidata client 0.42.0/bridge 0.4.0 porque o pacote também inclui
C139 já integrado. CI inclui 42 cenários no debug e nos ELFs extraídos do
`.deb`, ciclo 0.41→0.42→0.41→0.42, bridges XMQR/Mosquitto e imagem não root.
Aguardar todos os jobs da fonte exata antes de tag/release/DockerHub. Nenhuma
tag, release ou imagem foi publicada por este registro inicial. Não mover latest.

### Aceite final da fonte e0e553c e publicação

Os parágrafos anteriores registram as tentativas, não pendências atuais.
[CI final 39183999929](https://github.com/licodevone/xmqr-client/actions/runs/39183999929),
fonte **e0e553c3328839177c293870a6d12291c0099d19**, passou nos cinco jobs:
rust, ubuntu-package, bridge-integration, bridge-image e bridge-package-integration.
Inclui 193 testes Rust, 42 cenários de correções/controles no debug **e** nos
ELFs extraídos do pacote CI, migração real draft-v1→v2, regressões MQTT311/5,
TCP/WSS mTLS, bridges XMQR/Mosquitto, topologia gerenciada e ciclo de pacote.

O `.deb` foi baixado **dessa CI** (artifact 12157825242), não reempacotado de
outro commit. Package xmqr-client, Version 0.42.0-1, Architecture amd64;
SHA-256 **552db5568fde4c045df5c1e8bfe5d5c98a1a5073ac201ada218d977d1e8f54b8**.
Executáveis extraídos imprimem mqtt-client 0.42.0 e mqtt-bridge 0.4.0
(xmqr-client 0.42.0). Na máquina local, esse pacote passou novamente nos 42
cenários e no ciclo descartável Ubuntu26 0.41→0.42→0.41→0.42→purge,
preservando operator.keep e sem habilitar a unit: C143_PACKAGE_LIFECYCLE_PASS.

A imagem local foi construída da fonte limpa e0e553c (somente o target/
vendorizado preexistente ficou não rastreado e excluído do contexto). Os dois
binários imprimem versões corretas, usuário 10001:10001, Linux/amd64, revisão
OCI exata. O ELF mqtt-client extraído da imagem passou nos 42 cenários.
Também se executou o P232 **original sem alteração** nos ELFs da imagem e do
pacote: ambos deram 8 PASS/4 FAIL/0 BLOCKED, cinco obrigações corrigidas PASS
e apenas as quatro recomendações recebidas ainda FAIL. Não se declara PASS
global da auditoria. A imagem não instala o `.deb` Ubuntu dentro do Bookworm.

[Tag/prerelease v0.42.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.42.0)
publicadas após o aceite. Tag anotada **712dedb731c9322bd6f4f27a4e75cfa63050f4f8**,
commit alvo e0e553c completo acima, confirmado por ls-remote/peeled tag.
Docker Hub **lepinheiro100/xmqr-client:0.42.0**, digest remoto
**sha256:30ebb6d40cbe075f3ff68ae01b54a4f3d73154aa95fde3b65acbc92bf746f838**,
confirmado por imagetools, pull por digest e execução read-only/network-none.
Nenhum latest, tag ou asset antigo foi alterado.

Seis assets remotos baixados para dist/c143-remote-042 e comparados por
SHA-256 com dist/c143-release-e0e553c: todos idênticos. Além do pacote acima:

- Imagem docker-load tar.gz: SHA-256
  **aa81c87148c0ca5c14bf138d4478b66eb2188e4fd70f6ce272f83ade356fe503**.
- Manifesto: SHA-256
  **024b203060e00ff8622ad756ed525fc8e7cd1672381f2e1dc6ca7a837806f26a**.
- Os três sidecars também foram confrontados byte a byte por seus hashes.
  Além disso, sha256sum -c dos três sidecars baixados passou no WSL/Ubuntu.

[Evidência final de pacote/imagem/CI](../../docs/conformance/client-audit-fixes-release-0.42.0.json)
com fonte exata, relatórios completos e manifesto. O JSON local pré-commit
permanece separado, sem apagar os FAILs anteriores. Broker 0.43.0 e admin
independente 0.11.0 não foram alterados por C140–C143. Fase 8/T02 e as
recomendações recebidas continuam abertas. Esses commits documentais de
reconciliação posteriores não substituem a fonte dos artefatos e0e553c.

Sem instalação no host, sem modificar senha/certificados reais, firewall,
serviços, ESP32/COM4 ou estado do operador. Não houve homologação Windows
nativa, arm64, outras versões Ubuntu, crash de energia ou conformidade integral.
