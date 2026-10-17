# C96 — marco client 0.26.0 publicado

O [prompt C96](../clients/96-release-0.26.0.md) antecedeu a atualização de versão (`8f66ad7`). Fonte exata `bde591008309fb34cc47970d5c8253a162bc59d4`: 69 testes CLI e 65 wire, fmt e 7 contratos de distribuição locais PASS; [CI PASS](https://github.com/licodevone/xmqr-client/actions/runs/39020323105), inclusive pacote Ubuntu 26.04. Integração C95/P164 com TCP real passou; o ciclo descartável Ubuntu 26.04 com broker/client 0.28.0/0.25.0 → 0.29.0/0.26.0 → rollback → remoção retornou `PACKAGE_LIFECYCLE_PASS`, preservando arquivos privados.

Pacote `xmqr-client_0.26.0-1_amd64.deb`: SHA-256 `0808aa4d10303fb3a476e4d7a0987247231f4fa4935a1b0eedc273e3947ce2db`. A tag anotada `v0.26.0` aponta à fonte exata; a [prerelease](https://github.com/licodevone/xmqr-client/releases/tag/v0.26.0) tem 6 anexos com tamanho e SHA-256 remotos iguais aos locais (`CLIENT_REMOTE_HASH_PASS 6`). Imagem numerada `lepinheiro100/xmqr-client:0.26.0` executada pelo digest remoto `sha256:9f41374911340e67e1e1f6be63411666032eba79077d7e04b21b6e919ef22862`, com versão conferida.

WSL do operador e Docker `latest` permaneceram intactos. O broker 0.29.0 usa estado v15: rollback desse broker a 0.28.0 exige backup privado v14 anterior. Fases 6A–6D e 7–9 seguem abertas.
