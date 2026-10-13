# C42 — Distribuição Docker Hub e plano de evolução do client

Versão: 1.0. Data: 2026-10-13. Estado: execução autorizada.

Entrada: HEAD 6c54281, versão 0.8.0; preservar tar.gz/checksum 0.7.1
preexistentes e não commitados. Broker/admin segue P78. O usuário autorizou
commit/push/tag/prerelease/.deb/Hub e confirmou que quer concluir o plano
e a distribuição, não implementar todas as fases agora.

Objetivo: patch 0.8.1 de distribuição, label Docker corrigido (a versão
anterior do Dockerfile ainda era 0.7.1), contexto allowlist, build com
conferência da versão do binário, artefatos/checksums/proveniência e imagem
lepinheiro100/xmqr-client:0.8.1. Detalhar o backlog da CLI e suas dependências
do broker no plano compartilhado, sem importar o crate do broker.

Escopo: Cargo.toml/lock, Dockerfile/.dockerignore, scripts/test_distribution.py,
scripts/release_assets.py, docs/docker.md, docs/mosquitto-parity.md,
README/CHANGELOG, prompt e registro C42. Não alterar src/, rumqttc, wire,
TLS/SAN, filas, timeout, retained, senha oculta ou limites 1024/4096/65536.

Gates: fmt/test/Clippy/build release/inventário Linux, testes de contrato,
consulta da versão e integração mTLS com o broker em containers temporários;
.deb Ubuntu26 amd64 com lifecycle isolado, hashes e commit de origem.
Não instalar o .deb Ubuntu na imagem Bookworm. Não instalar no host ou WSL
do mantenedor, nem incluir arquivos privados. Confirmar digest depois de
push da imagem numérica, sem mover latest. Tag imutável v0.8.1 e prerelease
somente com artefatos aceitos, proveniência e limitações explícitas.

Planejamento não é implementação. Sem MQTT5/WS/perfis/repetição nesta fatia.
Registrar resultados reais em prompts/registros/C42-distribuicao-hub-plano.md;
revisar este prompt antes de corrigir falhas e não avançar fases por atalhos.
