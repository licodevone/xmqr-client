# C115 — distribuição experimental client independente 0.33.0

Entrada: client 0.32.0 publicado, C114 em validação em `main`; broker P212 aceito em `main` e P213 preparado. Este prompt precede mudança de versão, tag ou release.

Objetivo: publicar client 0.33.0 com pub/sub WSS mTLS local C114, depois de CI e integração cruzada aprovados. Preservar WS `open-lab`, TCP/TLS e os manifestos duráveis existentes. WSS continua sem perfil, outbox/assinatura durável ou destino remoto.

Escopo: Cargo/lock, `--version`, CHANGELOG, VERSIONING, README, nota e registro; pacote Ubuntu 26.04 amd64, export Docker numerado, manifesto/checksums e prerelease. Não alterar o admin independente, imagens `latest` nem WSL instalado do operador.

Gate: CI da fonte candidata exata em Rust e pacote Ubuntu; fixture WSS com CA/SAN/mTLS/subprotocolo, integração client ↔ broker P212 em MQTT 3.1.1/5 por WSS/TCP, negativos de senha/ACL/CA, regressão WS/TCP/TLS, fmt, suíte, Clippy, build, licenças e contratos. Validar SHA/metadados do `.deb`, ciclo descartável upgrade/rollback com broker0.40/client0.33, imagem por digest remoto e seis anexos remotos por tamanho/hash. Criar tag anotada `v0.33.0` e prerelease somente após os gates; documentar riscos, não declarar fase 7 global completa.
