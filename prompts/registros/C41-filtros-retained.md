# C41 — filtros retained

2026-10-12. [Prompt rev1/rev2](../clients/41-filtros-retained.md) criado antes do
código; revisão antes da correção Clippy. Base8bb7aaf/0.7.1; Docker archives
preexistentes preservados. Client passa a0.8.0, broker0.11.2 não alterado.

Flags sub --ignore-retained/--retained-only opcionais, exclusivas; --count conta
saída. Mensagens filtradas seguem ACKs QoS1/2. Default, ACL, SUBSCRIBE,
credenciais, TLS/SAN, limites, cancelamento e reconexão preservados.

PASS Windows inicial:30unit+21wire, build release. FAIL Clippy inicial: cast
usize→u8 na fixture. Revisão rev2 exige try_from; correção sem relaxar lints.
Tentativas de conexão WSL de login tiveram erro0x8007274c; retry sem perfil
resolveu. Não alterado/reiniciado nenhum serviço real para contornar o erro.

PASS Ubuntu26/WSL final0.8.0: fmt--check,30unit+22wire, clippy-Dwarnings e
build release. Arquivos target/c41-final-{tests,clippy,build}.log.
PASS integração C41: retained bootstrap/live, count e QoS0/1/2, dois filtros
com broker separado; target/c41-final-integration.log.
PASS regressão C27:4 testes, incluindo peer TLS CA/SAN/mTLS e negativos;
target/c41-c27.log. Fixtures wire verificam ACKs de mensagens filtradas e
DISCONNECT só após receber a mensagem exibida e concluir QoS2.

PASS pacote Ubuntu26amd640.8.0-1: build_deb nativo; lifecycle container
install0.7.1→upgrade0.8.0→rollback/remove/purge, preservação de arquivos/estado
fictício e units. Broker0.11.1→0.11.2 existente usado como par no harness,
sem nova releasebroker. Log ../xmqr/target/c41-package-lifecycle.log,
PACKAGE_LIFECYCLE_PASS. Nenhuma instalação do operador foi alterada.
Artefatos finais com README0.8.0 novamente passaram no lifecycle:
../xmqr/target/c41-package-final.log, PACKAGE_LIFECYCLE_PASS.

PASS Windows final0.8.0: fmt--check,30unit+21wire, clippy-Dwarnings e build
release (linker emite aviso informativo localizado, sem falha de gate).
ZIP portátil extraído, SHA do executável idêntico e --version0.8.0/exit0.
TAR Linux extraído, cmp idêntico e --version0.8.0/exit0. SHA-256 de deb/tar/zip
verificado. README distribuído foi atualizado para0.8.0 antes dos artefatos finais.

SHA-256 finais:

- deb: b90b7afea116931ee8c0265e1f15fb77db31410379e2a62d56323a8bba71e25f
- LinuxTAR: ccd14693ee3b79193f238aa11ed0c1acafaf5de8a7df77b9492b6a019eb0702e
- WindowsZIP: d4b9854c0671494f304ff29445f2e418ced38daf076971e8bd6296d47a57150a

Versão/tag/release experimental autorizadas após gates finais. Artefatos em
target/c41-assets; checksums e validação portátil registrados antes de publicação.
Sem prontidão de produção, MQTT5/WS/WSS, APT, arm64, Windowsbroker ou novo
aceite de MSRV1.88. Fase2 permanece aberta; filtros não encerram fase5.
