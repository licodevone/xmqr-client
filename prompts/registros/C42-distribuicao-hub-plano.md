# C42 — Aceite de fontes e candidatos de distribuição

Data 2026-10-13. PromptC42 versão1.0, base6c54281/client0.8.0. Patch0.8.1
somente distribuição: Dockerlabel corrigido, conferência da versão real,
contexto allowlist, exportador/checksums/proveniência e plano compartilhado.
Src/rumqttc/TLS/SAN/credenciais/QoS/filtros retained/limites inalterados.
Dois tar.gz/checksum0.7.1 preexistentes não serão incluídos nos commits.

Gates Ubuntu26/Rust1.99amd64: fmtPASS,54 testes (32 unitários+22wire)PASS,
Clippyall-targets-DwarningsPASS,releasebuildPASS,licençasPASS e5 contratos
PythonPASS. Cargo.lock mudou somente a versão do pacote próprio.

DockerBookwormcandidate0.8.1PASS, consulta de versão/UID10001/SIGTERM.
Teste real com broker/admin0.11.3: mTLSpub/subQoS1, bytes/retain, retained após
restart e CA/senha inválidas recusadasPASS. Read-only/capsALL removidas/
no-new-privileges; apenas volumes/redes/estado temporários.

DebUbuntu26amd64PASS. Lifecycle com broker em containerUbuntu26descartável:
install0.8.0,upgrade0.8.1,rollback,remove/purge e preservaçãoPASS.
Aviso dpkg-shlibdeps usr-merge não impediu instalação/execução.
Guia e plano distinguem .debUbuntu da imagemBookworm; não instalar um no outro.

Aceite conjunto e revisão do plano em [P78](https://github.com/licodevone/xmqr/blob/main/prompts/registros/P78-plano-paridade-distribuicao-hub.md).
Após commit limpo aceito, gerar artefatos finais desse commit, repetir gates,
publicar tag imutávelv0.8.1/prerelease/.deb/checksums/manifesto e imagem
lepinheiro100/xmqr-client:0.8.1. Este registro é anterior à publicação; os
anexos finais conterão evidência real e digest remoto, sem labels candidate.

Não mover latest/tags antigas. NOT_RUN: arm64, instalaçãoUbuntu24, MSRV1.88,
ESP32 físico, novo MQTT5/WS/perfis/repetição. Essas funções permanecem planejadas.
