# C82 rev1 — prerelease experimental client 0.21.0

Entrada: C80 `525d0d3f3e2fa07385498d185b1ca71c0bffa9cd` com [CI PASS](https://github.com/licodevone/xmqr-client/actions/runs/39000979892), C81 `dc2364f67ee22300bac0429d1df563d45e538425` aguardando CI; broker P144 aceito. Este prompt precede o bump. Exigir CI verde de C81.

Objetivo: publicar checkpoint client 0.21.0 com metadados PUBLISH na saída JSONL de `sub --protocol-version 5` e repetição sequencial MQTT 5 em `pub`. Atualizar Cargo/lock, Dockerfile, changelog, README e nota de release. O broker parceiro 0.23.0 conserva documento durável v10; nenhum fluxo QoS local é retomado pelo client. Fase 6C global aberta.

Aceite: gates locais e CI da fonte exata PASS; `.deb` Ubuntu26 amd64, imagem Docker numerada, ciclo de pacote conjunto install/upgrade/downgrade/remove em container descartável sem tocar a WSL do operador; tag anotada imutável `v0.21.0`, prerelease GitHub com manifesto/SHA-256 verificados e Docker Hub numerado com digest remoto, `latest` intacta. Sem ensaio físico ESP32.
