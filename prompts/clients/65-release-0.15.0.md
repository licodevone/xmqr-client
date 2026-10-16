# C65 rev1 — prerelease experimental client 0.15.0

Entrada: `main` `f68174b`, árvore limpa, versão publicada 0.14.0, C60–C64 implementados com gates locais e integração real PASS; CI de C64 pendente. Broker parceiro P123 em `621ac36`, versão publicada 0.16.0. Fases 6A–6D permanecem abertas.

Rev2: [CI C64](https://github.com/licodevone/xmqr-client/actions/runs/38983320426) PASS; broker P123 exigiu reparo separado P125, agora [CI `bb1084b`](https://github.com/licodevone/xmqr/actions/runs/38983737064) PASS. A base cruzada da release é `bb1084b` no broker e `f68174b` no client.

Objetivo: após CI de C64 PASS, publicar marco experimental 0.15.0 do client independente com QoS2 MQTT5, UNSUBACK, diagnósticos ACL e propriedades informativas de CONNECT. Elevar Cargo/lock e Dockerfile; atualizar changelog, README, paridade e nota de release com limites. Sem alterar instalação WSL do operador ou admin independente.

Aceite: gates fmt/test/Clippy/build e integração cruzada no commit exato, inventário MIT, contratos de distribuição, `.deb` Ubuntu26 amd64 sem instalação, imagem Docker numerada verificada, tag anotada imutável `v0.15.0`, prerelease GitHub com assets, SHA-256, manifesto, upgrade/rollback. Publicar Docker Hub só após digest remoto verificável; nunca mover `latest`. Registrar SHA, CI, anexos e riscos. Não anunciar conformidade MQTT5 integral ou produção.
