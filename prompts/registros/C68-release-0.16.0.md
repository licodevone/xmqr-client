# C68 — prerelease experimental do client 0.16.0

- [Prompt C68](../clients/68-release-0.16.0.md) criado antes do bump. Base funcional C67: [CI](https://github.com/licodevone/xmqr-client/actions/runs/38989868018) PASS; broker P128: [CI](https://github.com/licodevone/xmqr/actions/runs/38989862684) PASS.
- Fonte imutável: commit `bf1065b559e0c955378a6c464602088297229f83`, tag anotada `v0.16.0`. [CI da fonte](https://github.com/licodevone/xmqr-client/actions/runs/38990023887) PASS. Gates locais de fmt, 48 unitários/42 wire, Clippy, build, licenças, integração real e pacote Ubuntu26 PASS.
- [Prerelease](https://github.com/licodevone/xmqr-client/releases/tag/v0.16.0): pacote `.deb`, imagem Docker, dois arquivos SHA-256, manifesto e SHA-256 do manifesto (seis anexos). Nome, tamanho e digest SHA-256 dos seis anexos remotos conferidos contra os arquivos locais.
- Docker Hub numerado: `lepinheiro100/xmqr-client:0.16.0` digest `sha256:fd36c446bb9943c861c452cd7c4bc8384336ee71724a0cfcffc2adf8bea64e17`. Digest remoto conferido e versão executada por digest. `latest` não foi movida.
- Rollback e limites na [nota](../../docs/releases/0.16.0.md). Pacote não foi instalado no WSL do operador. Fases 6A–6D permanecem abertas.
