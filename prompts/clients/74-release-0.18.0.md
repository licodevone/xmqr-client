# C74 rev1 — prerelease experimental client 0.18.0

Entrada: `main` após C73, client publicado 0.17.0; fases 6A–6D abertas. Exigir CI verde de P136/C73 antes do bump.

Objetivo: publicar checkpoint 0.18.0 com C72 `--receive-maximum` e C73 `--maximum-packet-size` em `sub --protocol-version 5`. Bump Cargo/lock, Dockerfile, changelog, README e nota de release. Não alterar a instalação WSL do operador nem o admin independente.

Aceite: gates da fonte exata, integração com broker P136/Mosquitto e CI PASS; `.deb` Ubuntu26 amd64, imagem Docker numerada, checksums/manifesto verificados; tag anotada imutável `v0.18.0`, prerelease GitHub e digest Docker Hub remoto conferido, `latest` intacta. Documentar que novo processo ainda não retoma QoS próprio; fase 6 continua aberta.
