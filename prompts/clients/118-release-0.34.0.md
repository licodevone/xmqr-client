# C118 — distribuição experimental client 0.34.0

Entrada: client 0.33.0 publicado; C116 perfil WSS local aceito em `main`; C117 adiciona opt-in remoto WSS mTLS com teste Docker PASS. CI da fonte funcional ainda em execução ao abrir este prompt. Broker P214 é o par remoto. Este prompt precede mudança de versão, tag ou release.

Objetivo: publicar client independente 0.34.0 com C116/C117, preservando TCP/TLS e WS laboratório, sem migração dos manifestos duráveis. URL remota exige flag explícita; CA/SAN e certificado cliente permanecem obrigatórios. Sem navegador ou estado durável por URL.

Escopo: manifests/lock, `--version`, CHANGELOG, VERSIONING, README, nota de release, registro; pacote Ubuntu 26.04 amd64 e imagem Docker numerada. Não alterar WSL instalado do operador ou tags/`latest` anteriores.

Gate: CI da fonte candidata exata em Rust e pacote Ubuntu; perfil e CLI por WSS remoto/local 3.1.1/5 com broker P214 em rede descartável, negativos de opt-in/CA/SAN/senha/ACL, fmt, suíte, Clippy, licenças/distribuição. Metadata/SHA-256 do `.deb`, upgrade/rollback conjunto 0.33 → 0.34, imagem por digest remoto, tag anotada, prerelease e hashes de todos os anexos baixados. Registrar evidências sem fechar fase 7 global nem fases 6A–6D/8–9.
