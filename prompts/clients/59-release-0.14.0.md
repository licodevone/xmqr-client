# C59 rev1 — marco experimental MQTT 5 0.14.0

Data 2026-10-16. Entrada `9aa86ae` em `main`, client 0.13.0, C57/C58 aceitos localmente; CI remota pendente. Minor própria para pub/sub MQTT 5 QoS0/1 em sessão limpa, sem encerrar a fase6A.

Após CI de C58 aprovada, elevar Cargo/lock e Dockerfile para 0.14.0; atualizar changelog e nota de release. Validar fmt/test/Clippy/build, inventário, interop cruzada, contratos de distribuição e `.deb` Ubuntu26 amd64. Criar commit limpo, testar o commit exato e tag anotada imutável `v0.14.0`. Publicar prerelease GitHub com pacote, checksums e upgrade/rollback se os gates passarem; imagem Docker numerada somente se construída e inspecionada. Não mover `latest` ou tags anteriores; não anunciar conformidade MQTT 5 total/produção. Registrar SHA, CI, assets e limites.
