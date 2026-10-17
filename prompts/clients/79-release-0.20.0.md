# C79 rev1 — prerelease experimental client 0.20.0

Entrada: C77/C78 publicam metadados MQTT 5 em `main`; client publicado 0.19.0. Exigir CI verde da fonte C78 e do broker P142 antes do bump. Este prompt precede qualquer alteração de versão.

Objetivo: publicar checkpoint 0.20.0 do client com `--payload-format`, `--content-type`, `--response-topic`, `--correlation-data-hex` e `--publish-user-property` no `pub --protocol-version 5`. Atualizar Cargo/lock, Dockerfile, changelog, README e nota de release. O assinante ainda não expõe metadados; o próprio client não restaura fluxos QoS em andamento. Broker parceiro 0.22.0 grava documento v10 e exige backup privado v8 para rollback ao 0.21.0.

Aceite: gates locais, integração real com broker P142 e Mosquitto, CIs PASS; `.deb` Ubuntu26 amd64, imagem Docker numerada, upgrade/downgrade/remove em contêiner descartável sem tocar instalação WSL do operador; tag anotada imutável `v0.20.0`, prerelease GitHub com manifesto e SHA-256 conferidos, Docker Hub numerado com digest remoto e `latest` intacta. Checklist somente após evidência. Fases 6A–6D e 7–9 ainda abertas; sem ensaio físico ESP32.
