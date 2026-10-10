# C31 — systemd e Ubuntu26

2026-10-10. Prompt:../clients/31-systemd-ubuntu.md rev1.
Base3e7c16a/tagv0.4.0, árvore limpa. Escopo e commit/push/prereleases autorizados.
Versão nova0.5.0; tags herdadas0.1/0.2 (manifest0.7) permanecem imutáveis.

SIGTERM/SIGINT usam cancelamento existente: DISCONNECT limitado1s em conexão
disponível, sem reconectar/republicar. Sub template com conta própria, arquivo
de senha0600/donoUID, TLS/mTLS, sessão estável e restart limitado; pub oneshot
sem restart. Sem novo estado persistente ou garantia de retryQoS2 recebido.
Deb Ubuntu26 amd64, depende de ABI calculada, conta sem login, exemplos e
units opcionais; nada ativa automaticamente. Configs externas são preservadas.
CI fmt/test/Clippy/build/inventário e build/instalação/verificação deb emUbuntu26.

## PASS

- Ubuntu26.04/WSL Rust1.99:fmt --check,test --locked --offline (44 testes:
  28 unitários+16 wire),Clippy all-targets -Dwarnings,build release.
- Novo teste wire SIGTERM real observa DISCONNECT e exit0, sem reconectar.
- license_inventory --check PASS; Windows check all-targets PASS.
- Build deb/revisão teste2 no Ubuntu26 PASS. Docker Ubuntu26 limpo instala
  pelo apt, verifica units, atualiza revisão1→2, remove/purge sem perder
  credenciais externas. Fonte do teste compartilhado:../xmqr/packaging/test_package.sh.
- Integração systemctl/mTLS com broker independente e usuário não root no WSL:
  sub/pub oneshot,reload/restart broker,stop sub com exit0. Script isolado:
  ../xmqr/scripts/verify_systemd.py --test-user licod26.
- Broker:3 testes shutdown+9 regressõesWill PASS; C29 integração1 cenário PASS.

## FAIL inicial, corrigido

Teste de --version hardcoded0.4 falhou após bump: usa CARGO_PKG_VERSION agora.
Fixture systemd tinha ClientID acima23 caracteres e marcador JSON errado;
corrigida e rerodada com sucesso. Nenhuma falha final conhecida nesses gates.

## Limitações

Deb Ubuntu26 amd64; não validar Ubuntu24/arm64/MSRV1.88 exato por inferência.
Units testadas com paths/usuário temporários preservando hardening; nenhuma
instância permanente configurada no WSL. Sem APT próprio assinado, timers,
instalação Windows, persistência MQTT local ou prontidão para produção.
Broker seguro testado com client próprio; não equivale a TLS externo Mosquitto.
Commit/push/prerelease autorizados, feitos após gates e anunciados no resultado.

## Publicação / CI confirmados

Commit de implementação:2615908; tag anotada v0.5.0 sincronizada.
Prerelease:https://github.com/licodevone/xmqr-client/releases/tag/v0.5.0
Deb e checksum anexados e SHA256 remoto igual ao local.
GitHubActions:38085654031 — jobs rust e ubuntu-package PASS.
Repositório sincronizado; nenhuma instância permanente instalada no WSL.
