# C103 — marco experimental client 0.28.0

Entrada: client 0.27.0 publicado; C101 mostra o motivo de DISCONNECT remoto sem texto não confiável, e C102 envia User Property em SUBSCRIBE/UNSUBSCRIBE com limites, teste de fio, broker real e Mosquitto 2.0.22. Este prompt antecede qualquer mudança de versão.

Objetivo: distribuir client independente 0.28.0 em Cargo/lock, Dockerfile, README, CHANGELOG, VERSIONING e nota de release. Documentar exemplos de flags e a exigência de `--count` para propriedades de UNSUBSCRIBE. O broker parceiro 0.33.0/P182 conterá P180/P181. Admin independente fica em 0.10.0. Fluxo QoS local do client, fases 6A–6D e 7–9 permanecem abertos.

Aceite: CI do commit fonte exato após Rust/fmt/Clippy/build release/licença/distribuição, wire C101/C102, integração broker e Mosquitto; pacote Ubuntu 26.04 amd64 com upgrade 0.27.0 → 0.28.0 → rollback em container descartável, configuração privada preservada; imagem numerada executada por digest; tag anotada e prerelease com manifesto, tamanhos e SHA-256 remotos. Não mover Docker `latest` nem instalar no WSL do operador. Falhas exigem revisão do prompt antes de correção.
