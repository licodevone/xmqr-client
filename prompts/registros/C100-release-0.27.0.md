# C100 — marco client 0.27.0 publicado

O [prompt C100](../clients/100-release-0.27.0.md) antecedeu a atualização de versão (`da35d24`). Fonte exata `a3014fd66ada4733f169c4757f320b7ec4512b6a`: 76 testes CLI e 65 wire, fmt, Clippy, build release, licenças e 7 contratos de distribuição PASS localmente; [CI PASS](https://github.com/licodevone/xmqr-client/actions/runs/39022714234), inclusive pacote Ubuntu 26.04. A integração C97–C99/P166 com TCP real retornou `P166_C97_C98_C99_ASSIGNED_ID_PASS`. O ciclo descartável Ubuntu 26.04 broker/client 0.29.0/0.26.0 → 0.30.0/0.27.0 → rollback → remoção retornou `PACKAGE_LIFECYCLE_PASS`, preservando arquivos privados.

Pacote `xmqr-client_0.27.0-1_amd64.deb`: SHA-256 `40c2cc0c71ced201538b9dda98fd497eb53668cc2ca3036d90ad1d1f23b0836d`. A tag anotada `v0.27.0` aponta à fonte exata; a [prerelease](https://github.com/licodevone/xmqr-client/releases/tag/v0.27.0) tem 6 anexos com tamanho e SHA-256 remotos iguais aos locais (`CLIENT_REMOTE_HASH_PASS 6`). A imagem numerada `lepinheiro100/xmqr-client:0.27.0` foi executada pelo digest remoto `sha256:71a3d5655072695d7a7b96c3aeeec7d5cdea413d6bef1cb8ad9e8ad990f05d68`, com versão conferida.

A CLI pode reutilizar ID atribuído e criar sessão durável, mas não guarda estado QoS local. WSL do operador e Docker `latest` permaneceram intactos. Fases 6A–6D e 7–9 seguem abertas.
