# C71 rev1 — prerelease experimental client 0.17.0

Entrada: `main` após C70, client publicado 0.16.0; fases 6A–6D abertas.

Objetivo: publicar checkpoint 0.17.0 com C69 Message Expiry e C70 Will Delay MQTT 5. Bump Cargo/lock, Dockerfile, changelog, README e nota de release, sem alterar instalação WSL nem admin independente.

Aceite: CI funcional verde e gates da fonte exata; pacote Ubuntu26, imagem Docker numerada e checksums/manifesto verificados; tag anotada v0.17.0, prerelease GitHub e digest Docker Hub remoto conferido, sem mover `latest`. Documentar que novo processo ainda não retoma fluxo QoS próprio; broker P132 usa estado v8 e rollback exige backup anterior. Fase 6 permanece aberta.
