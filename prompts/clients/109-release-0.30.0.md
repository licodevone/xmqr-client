# C109 — marco experimental client 0.30.0

Entrada: client independente 0.29.0 publicado, C107 (outbox QoS 2) aceito e C108 (manifesto de assinatura MQTT 5) em validação. Este prompt precede qualquer mudança de versão 0.30.0; C109 não avança até C108 ter CI da fonte exata PASS e registro de aceite. Broker parceiro publicado 0.35.0; admin independente 0.10.0.

Objetivo: distribuir C107/C108 em 0.30.0. Atualizar Cargo/lock, Dockerfile, CHANGELOG, README, VERSIONING e nota de release. Documentar comandos, retomada por PUBLISH/DUP versus PUBREL, manifesto `Prepared`/`Active`, diretórios privados, limites de sessão e ausência de garantia de processamento por consumidor. MQTT 3.1.1 permanece padrão. Fases 6A–6D/7–9 e M03 continuam abertas; não alterar broker/admin nem migrar o estado do broker.

Aceite da fonte: fmt, suíte completa, Clippy, build release Linux, check Windows, licenças, sete contratos de distribuição, fixtures C105/C107/C108 e integrações broker real com restart e entrega única/assinatura retomada. CI do commit fonte exato, inclusive pacote Ubuntu 26.04 amd64 e units, antes de tag/release.

Aceite da distribuição: `.deb` 0.30.0-1, metadados/SHA-256, upgrade 0.29.0 → 0.30.0 → rollback descartável com arquivos privados preservados. Imagem Docker numerada 0.30.0, usuário não privilegiado, versão/labels, digest remoto executado; tag anotada imutável `v0.30.0` e prerelease GitHub com manifestos/anexos, tamanhos e hashes remotos verificados. Não mover `latest`, instalar no WSL do operador nem publicar APT. Registrar tentativa falha e repetir gate essencial quando necessário.

Evidência: registro C109 com hash da fonte/tag, CI, checksums, digest, ciclo de pacote, URLs e limites. Marcar checklist após publicação verificada. Admin 0.10.0 e broker 0.35.0 permanecem até seus marcos próprios.
