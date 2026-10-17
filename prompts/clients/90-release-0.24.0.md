# C90 rev1 — marco experimental client 0.24.0

Entrada: C89 valida `$share/grupo/filtro` MQTT 5 e passou contra Mosquitto 2.0.22; P153/P154 broker aceitaram ou estão em CI. Este prompt precede o bump de versão.

Objetivo: distribuir C89 no client 0.24.0, com documentação clara da disponibilidade do broker 0.25.0 (não oferece grupos) e 0.26.0 (grupos e sessão persistente quando publicada). Atualizar Cargo/lock, Dockerfile, changelog, README, VERSIONING, plano/nota, sem alterar contratos de pub/sub MQTT 3.1.1. Não afirmar retomada automática de fluxo QoS do processo do client nem AUTH.

Aceite: Rust CLI/wire, fmt, Clippy, build release, licenças, contratos de pacote e CI do commit exato; Mosquitto 2.0.22 e broker P154 em TCP descartável. Pacote Ubuntu26 amd64, imagem numerada, ciclo descartável install/upgrade/rollback/remove, tag anotada `v0.24.0`, prerelease com SHA-256/tamanhos verificados e Docker Hub digest remoto. Sem `latest`, instalação no WSL do operador ou teste ESP32.
