# C138 — candidata client 0.41.0 / bridge 0.3.0

Prompt [C138 v1.1](../clients/138-release-0.41.0-bridge-mqtt5-retain.md)
foi publicado antes da alteração de versão. Base C137 `9a02631` com
[CI completa](https://github.com/licodevone/xmqr-client/actions/runs/39169197510)
PASS. O `.deb` antigo 0.40.0-1 foi baixado da release publicada e seu
SHA-256 `5f453fbc17e0fbc919f457877176dd3149bf041f4aca565f2d9028d1f07a0fe6`
conferido; não se usou o pacote local diferente que já existia em `dist/`.

## PASS local da candidata

- Ubuntu 26.04 WSL: `cargo fmt --check`, `cargo test --locked --offline`,
  Clippy debug/release `--all-targets -Dwarnings`, build release dos dois
  binários e inventário MIT: PASS.
- Pacote nativo `xmqr-client_0.41.0-1_amd64.deb` com campos `Package`,
  `Version` e `Architecture` corretos; binários release imprimem
  `mqtt-client 0.41.0` e `mqtt-bridge 0.3.0 (xmqr-client 0.41.0)`.
  SHA-256 do pacote **local**: `0cfa8de438c1bfab3ae9ca776d9f581c1d8a44f3f87aa7fb75ce4147740fb01f`.
- Contêiner Ubuntu descartável: instalação 0.40→0.41→0.40→0.41,
  configuração do operador preservada e unit não habilitada:
  `C138_PACKAGE_LIFECYCLE_PASS`.
- Binários extraídos do `.deb` local: XMQR↔XMQR MQTT 5 QoS 0/1,
  XMQR↔Mosquitto em ambos sentidos e regressão QoS 2: PASS. Os checkpoints
  de crash presentes apenas em debug foram pulados explicitamente no
  binário release; CI C137 debug os executou.

## FAIL preservado e limite

Primeira e segunda tentativas de todo o conjunto de testes do pacote:
`docker run` retornou 125 ao iniciar o contêiner Mosquitto no segundo
sentido, antes de exercitar a bridge; a fixture antiga não imprimia o
`stderr` capturado. C138 v1.1 adicionou diagnóstico somente à partida do
listener, sem imprimir o segredo do comando de hash. Quatro repetições
posteriores do cenário passaram, mas a causa das duas falhas iniciais
permanece indeterminada. Isto não é PASS retroativo nem prova de ausência
de flake no CI.

`dpkg-shlibdeps` advertiu sobre diversions da libc6 do WSL; o pacote
construiu e passou no contêiner nativo. O hash do pacote local não prova
reprodutibilidade bit a bit do pacote da CI.

A [CI da candidata `eb017e3`](https://github.com/licodevone/xmqr-client/actions/runs/39170168298)
passou nos cinco jobs, incluindo pacote e imagem em checkout limpo. O
build Docker **local** do mesmo código falhou duas vezes enviando contexto
de 134 MB (`context canceled`); o `target/` não rastreado do crate
vendorizado ocupa 639 MB. Após C138 v1.2 e exclusão explícita no
`.dockerignore`, o build local terminou PASS sem remover a saída local.
Os dois FAILs ficam preservados; a causa exata do cancelamento não foi
provada, mas a exclusão reduz o contexto e elimina essa superfície.

CI da fonte final com `.dockerignore`, digest remoto, tag, anexos e
publicação ainda pendentes. A fase 8/T02 permanece aberta.
