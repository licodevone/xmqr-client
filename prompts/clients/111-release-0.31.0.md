# C111 — marco experimental client 0.31.0

Entrada: client 0.30.0 publicado; C110 aceito em `main` após a tag. Broker parceiro 0.37.0 e admin independente 0.10.0 publicados. Este prompt precede qualquer alteração de versão 0.31.0.

Objetivo: distribuir C110, que confirma PUBLISH QoS 1/2 de uma assinatura persistente após escrita/flush em stdout e não envia ACK quando a saída falha. Atualizar Cargo/lock, Dockerfile, CHANGELOG, README, VERSIONING e nota de release. Distinguir a melhoria da ausência de journal inbound e da impossibilidade de garantir processamento exatamente uma vez. MQTT 3.1.1 permanece padrão; fases 6A–6D/7–9 e M03 abertas.

Aceite da fonte: fmt, suíte, Clippy, build release Linux, check Windows, licenças, sete contratos de distribuição, fixtures C105/C107/C108/C110 e broker real com restart/reeentrega. CI da revisão exata, inclusive pacote Ubuntu 26.04 amd64 e units, antes da tag/release.

Aceite da distribuição: `.deb` 0.31.0-1, metadata/SHA-256, upgrade 0.30.0 → 0.31.0 → rollback descartável com arquivos privados preservados. Imagem Docker numerada, usuário não privilegiado, versão/labels, digest remoto executado; tag anotada imutável `v0.31.0` e prerelease GitHub com seis anexos, tamanhos/hashes remotos conferidos. Não mover `latest`, instalar no WSL do operador nem publicar APT. Registrar desvios e repetir gates essenciais. Atualizar índice local/curso após publicação.
