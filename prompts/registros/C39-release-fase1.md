# C39 — aceite/release0.7.0

Base35580ba/version0.6.0. PromptC39 e revisãoC38rev5 antes das alterações.
Testewire compara comprimento da saída comzero, sem exposição de conteúdo,
resolvendo Clippy1.99 Ubuntu. Sem mudança de protocolo/retry/TLS/dependências.
Manifests/lock0.7.0, README/changelog e matriz atualizados. Tag0.6 preservada.

Runner final broker target/phase1-20261011T232424Z-scjb4yeg:52 testesclient
(31unit+21wire), fmt/Clippy/build/inventárioPASS; C27 e C38 reaisPASS. Broker
103testes e integração independenteMosquittoPASS. Fontes antes/depois iguais.
Binário otimizado0.7.0 e mTLS/systemd temporárioPASS. PackageLifecyclePASS:
instalação0.5.0, upgrade0.7.0, rollback, remove/purge, docs/credencial fictícia
preservados em containerUbuntu26. Logs no target/phase1-release-assets do broker.

Deb xmqr-client_0.7.0-1_amd64.deb SHA-256
fb216ddb47ad24d2870e3f7a8e8c5f0b55b47e946e14148d53394b8f74ecdc92.
Tagv0.7.0/prerelease autorizadas após aceite combinado; deb/checksum/resumo.
Registro conjunto: [P61](../../../xmqr/prompts/registros/P61-aceite-final-fase1.md).

Sem alteração da instalação real. Inboundrumqttc cap65531 com reserva5 mantido;
frame canônico65536 pode ser recusado por1byte. Não homologar Ubuntu24/arm64 ou
MSRV1.88 exato. Gates Windows históricos não foram repetidos nesta entrega;
não publicar pacoteWindows novo com base no artefato antigo0.6.0.
