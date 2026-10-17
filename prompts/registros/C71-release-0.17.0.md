# C71 — prerelease experimental client 0.17.0

- [Prompt C71](../clients/71-release-0.17.0.md) criado antes do bump. Base C70 `f64ddd9` com [CI PASS](https://github.com/licodevone/xmqr-client/actions/runs/38991792436); broker P132 com [CI PASS](https://github.com/licodevone/xmqr/actions/runs/38992393173).
- Fonte imutável: commit `ebf1b932a83581e597b54282c8deb29f34ab55c0`, tag anotada `v0.17.0`, [CI da fonte PASS](https://github.com/licodevone/xmqr-client/actions/runs/38992583146). Gates locais fmt, 50 testes unitários/44 wire, Clippy, build, licenças e contratos de distribuição PASS.
- [Prerelease](https://github.com/licodevone/xmqr-client/releases/tag/v0.17.0): `.deb`, imagem Docker, dois SHA-256, manifesto e SHA-256 do manifesto (seis anexos). Tamanho e SHA-256 remotos conferidos.
- Docker Hub numerado `lepinheiro100/xmqr-client:0.17.0`, digest `sha256:06b16d448ef8fc03940e4235ff8a9b06e560212bc439a327cbaa8d60804e9e1a` remoto conferido; versão executada por digest. `latest` intacta.
- Pacotes antigo/novo e downgrade/removal passaram no contêiner descartável Ubuntu26 (`PACKAGE_LIFECYCLE_PASS`), sem instalar no WSL do operador. Limites e rollback na [nota](../../docs/releases/0.17.0.md). Fases 6A–6D abertas.
