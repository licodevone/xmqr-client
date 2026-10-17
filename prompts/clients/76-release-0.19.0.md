# C76 rev1 — prerelease experimental client 0.19.0

Entrada: C75 entrega Topic Alias Maximum e reconstrução da saída em `main`; client publicado 0.18.0. Exigir CI verde da fonte C75 e do broker P139 antes do bump. Este prompt precede qualquer alteração de versão.

Objetivo: publicar checkpoint 0.19.0 do client com `sub --protocol-version 5 --topic-alias-maximum 0..16`. Atualizar Cargo/lock, Dockerfile, changelog, README e nota de release. Explicitar que o client não comprime publicações, não retoma automaticamente a assinatura no mesmo processo e não restaura fluxos QoS próprios. O broker parceiro 0.21.0 mantém documento durável v8.

Aceite: gates locais da fonte exata, integração com broker P139/C75 e CI PASS; `.deb` Ubuntu26 amd64, imagem Docker numerada, ciclo upgrade/downgrade/remove em contêiner descartável sem instalar no WSL do operador; tag anotada imutável `v0.19.0`, prerelease GitHub com manifesto/SHA-256 conferidos, Docker Hub numerado com digest remoto e `latest` intacta. Documentar incompatibilidade do `mosquitto_sub` 2.0.22 na recepção de alias abreviado, sem reivindicar essa direção. Fase 6C e demais fases 6–9 seguem abertas.
