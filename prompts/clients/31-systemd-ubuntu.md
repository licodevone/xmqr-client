# C31 rev.1 — systemd, Ubuntu e release

Base main 3e7c16a, tag v0.4.0, árvore limpa. Implementação, commit/push e
release autorizados. Alvo Ubuntu26.04/WSL amd64 solicitado pelo usuário.
Versão 0.5.0 própria; preservar tags anteriores e proveniência ORIGIN.json.

Escopo: tratar SIGTERM/SIGINT com o fluxo existente de DISCONNECT limitado;
unit opcional de assinatura e exemplo oneshot de publicação; deb, CI,
docs/readme/changelog e registro. Preservar rumqttc, limites, TLS/SAN,
senha em arquivo dono do UID e modo0600. Não reenviar publicações incertas,
nem habilitar/publicar dados automaticamente. Sem nova persistência/retry.

Arquivos: src/main.rs/session.rs/signals.rs, tests/validation, packaging,
scripts, .github/workflows, README/CHANGELOG/docs. Sem importar broker.
Aceite: fmt/test/clippy/build WSL; SIGTERM real em fixture e broker isolados;
deb Ubuntu26 amd64 e instalação/upgrade/remove isolados. Sem credenciais em
pacote/log/git. Serviço contínuo tem restart limitado, publicação não reinicia.
Publicar prerelease/deb/checksum após validação. APT próprio aguarda destino/chave.
