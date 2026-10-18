# C113 — marco experimental client 0.32.0

Entrada: client 0.31.0 publicado; C112 aceito em `main` após a tag. Broker P210 WebSocket em validação; broker 0.38.0 publicado. Este prompt precede qualquer alteração de versão 0.32.0.

Objetivo: distribuir C112, `--ws-url` somente em loopback com `--open-lab` para `pub`/`sub` MQTT 3.1.1/5. Atualizar Cargo/lock, Dockerfile, CHANGELOG, README, VERSIONING e nota. Deixar explícito que WSS, perfis e estado durável sobre WS não foram implementados e que o broker publicado só oferece WS a partir de 0.39.0 se esse marco for aceito.

Aceite da fonte: fmt, suíte, Clippy, build release Linux, check Windows, licenças, sete contratos de distribuição, fixture C112 e integração broker P210; CI da revisão exata, inclusive pacote Ubuntu 26.04 amd64 e units, antes da tag/release. Aceite da distribuição: `.deb` 0.32.0-1, metadata/SHA-256, upgrade 0.31.0 → 0.32.0 → rollback descartável com arquivos privados preservados. Imagem Docker numerada, usuário não privilegiado, versão/labels, digest remoto executado; tag anotada imutável `v0.32.0` e prerelease GitHub com seis anexos, tamanhos/hashes remotos conferidos. Não mover `latest`, instalar no WSL do operador nem publicar APT. Registrar desvios e repetir gates essenciais. Atualizar índice local/curso após publicação.
