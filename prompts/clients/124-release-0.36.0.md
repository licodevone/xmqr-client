# C124 — distribuição experimental client 0.36.0

Entrada: client 0.35.0 publicado; C122/C123 aceitos em `main` para outbox QoS 2 e manifesto de assinatura persistente por WSS mTLS. Broker 0.42.0/P216 é o par publicado. Este prompt precede versionamento, tag ou release.

Objetivo: distribuir client independente 0.36.0, sem migração dos journals/manifestos TCP e com URL WSS canônica como identidade. WS aberto e estado local recebido continuam fora do contrato; fase 7 global permanece aberta.

Escopo: manifest/lock, versão CLI, Dockerfile, changelog/README/versionamento, pacote Ubuntu 26.04 amd64, imagem Docker numerada, nota e registro de prerelease. Não alterar WSL instalado, tags antigas nem `latest`.

Gate: CI da fonte exata (Rust/pacote), fixture WSS de PUBREC/PUBCOMP, SUBACK incerto, Session Present e ACK após flush, integração com broker 0.42.0, regressões TCP, `.deb` SHA-256/metadata, upgrade/rollback descartável 0.35.0 ↔ 0.36.0, imagem por digest remoto, tag imutável, prerelease e hashes remotos dos anexos. Registrar PASS/FAIL/BLOCKED honestamente.
