# C93 rev1 — marco experimental client 0.25.0

Entrada: C91 implementa SASL PLAIN no CONNECT MQTT 5 sobre TLS, e C92 oferece `--reauth-before-io` com AUTH `0x19` e espera AUTH `0x00` antes de pub/sub. O broker parceiro P159/P160 ainda depende do marco P161. Este prompt precede qualquer mudança de versão.

Objetivo: distribuir C91/C92 no client 0.25.0. Atualizar Cargo/lock, Dockerfile, changelog, README, VERSIONING e nota de release. Documentar a cópia identificada de `rumqttc` 0.25.1 e sua licença/patch, os limites de reautenticação explícita única e a ausência de desafio AUTH, rotação em serviço e fluxo MQTT 5 completo. Não alterar o admin independente, que continua 0.9.0.

Aceite: wire fake TLS, broker P160 real em TLS/mTLS com pub/sub após AUTH, regressão C91; testes CLI/wire, fmt, Clippy, build release, inventário de licenças, contratos e CI do commit exato; pacote Ubuntu 26.04 amd64 e ciclo descartável de upgrade/rollback; imagem numerada e execução por digest; tag anotada e prerelease com manifesto, SHA-256/tamanhos remotos conferidos. Sem mover `latest`, instalar no WSL do operador ou declarar fases 6A–6D concluídas.
