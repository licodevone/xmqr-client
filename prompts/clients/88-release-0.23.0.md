# C88 rev1 — marco experimental client 0.23.0

Entrada: C87 Subscription Identifier aceito com [CI PASS](https://github.com/licodevone/xmqr-client/actions/runs/39007425763); broker P150/estado v12 em validação. Este prompt precede o bump de versão.

Objetivo: publicar client 0.23.0 com `--subscription-identifier` MQTT 5 e IDs no JSONL. O parceiro XMQR precisa de P150/0.25.0; Mosquitto 2.0.22 já foi validado. Atualizar Cargo/lock, Dockerfile, changelog, README e nota, sem declarar MQTT 5 integral nem replay automático do fluxo QoS local.

Aceite: gates locais e CI do commit exato, `.deb` Ubuntu26 amd64, imagem numerada, ciclo de instalação/upgrade/downgrade/remove em contêiner descartável com broker 0.24→0.25; tag anotada `v0.23.0`, prerelease GitHub com manifesto/SHA-256 remotos conferidos e Docker Hub numerado com digest remoto. `latest` intacta; sem alterar instalação WSL do operador ou executar teste físico.
