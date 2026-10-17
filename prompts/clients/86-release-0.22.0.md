# C86 rev1 — marco experimental client 0.22.0

Entrada: C83–C85 aceitos, incluindo C85 `f3797bdc2e0aa932ad526c3f4586b244451af93c` com CI PASS; P148 broker aguarda CI. Este prompt precede o bump de versão.

Objetivo: publicar client 0.22.0 com Topic Alias compacto no publisher e opções No Local/Retain As Published no assinante MQTT 5. Atualizar Cargo/lock, Dockerfile, changelog, README e nota de release. Registrar que o broker parceiro precisa de P148/0.24.0 para os bits de assinatura; Mosquitto 2.0.22 já foi homologado para o client. Não declarar suporte MQTT 5 integral nem retomada automática de QoS local.

Aceite: gates locais e CI do commit exato, `.deb` Ubuntu26 amd64, imagem numerada, ciclo de instalação/upgrade/downgrade/remove em contêiner descartável; tag anotada `v0.22.0`, prerelease GitHub com manifesto/SHA256 remotos conferidos e Docker Hub numerado com digest remoto. `latest` intacta; sem alterar instalação WSL do operador ou executar teste físico.
