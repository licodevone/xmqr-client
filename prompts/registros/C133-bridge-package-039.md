# C133 — integração e publicação do pacote da bridge

Prompt `10d520b` precedeu a integração `bfa0581` e o gate de exportação
`2a23522` na branch `release/0.39.0-bridge`, a partir da tag 0.38.0
`fca6fc8`. A versão 0.37.0 candidata de C128 não foi publicada depois
da 0.38.0; C128 foi integrado como 0.39.0, sem retirar C129.

## PASS

- Fonte exata `2a2352289177e6173c901d99c762b112d2083cd6`:
  [CI 39154151247](https://github.com/licodevone/xmqr-client/actions/runs/39154151247)
  PASS nos jobs Rust/C129, bridge-integration, ubuntu-package (incluindo
  upgrade/rollback 0.38→0.39→0.38) e bridge-image.
- WSL Ubuntu 26.04: `cargo fmt --check`, `cargo test --locked` (2 testes
  bridge, 110 unitários client, 1 WebSocket e 68 wire), `cargo clippy
  --locked --all-targets -- -D warnings` e `cargo build --locked --bins` PASS.
- `.deb` da CI: SHA256
  `d2b896ea986628f6b49acab14500b9e900ad4654f7be5b05605c3c02d3460b86`.
  Lifecycle descartável Ubuntu 26 PASS, com configuração preservada e unit
  opt-in. Binários extraídos desse `.deb` passaram na fixture TLS/mTLS QoS 1
  com dois brokers, incluindo negativos CA/SAN/senha/ACL e reconexão, e na
  topologia de duas bridges/três brokers com recusa de ciclo.
- Imagem local validou as versões dos dois binários, usuário 10001:10001 e
  revisão OCI; push numerado com digest remoto
  `sha256:d38e5c8d34ee6795b378f8c619bcad4a1cfad343f4e10501d60deeaa703b4530`.
- [Tag e prerelease v0.39.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.39.0)
  da fonte exata; seis anexos remotos baixados e três hashes SHA256 PASS.

## FAIL e limites

- Um comando local que encadeou o lifecycle Docker e as fixtures WSL passou
  no lifecycle, mas a fixture seguinte não iniciou por erro transitório
  `FileNotFoundError` do diretório corrente da montagem `/mnt/d`. Em nova
  invocação WSL, as duas fixtures com o mesmo `.deb` da CI passaram. Essa
  falha não é tratada como resultado de protocolo.
- O `.deb` compilado localmente tinha hash diferente do `.deb` da CI. A
  release usa exclusivamente o `.deb` da CI, que foi repetido nos ensaios
  locais e tem hash remoto verificado; não há alegação de build bit a bit
  reproduzível.
- O checkpoint de crash após PUBACK/destino da bridge usa marcador apenas
  no build de debug; a fixture de pacote release o pula explicitamente.
  A CI de integração em debug exerce o caminho pertinente. Sem garantia de
  entrega única, proteção universal de loops, QoS 2 de bridge ou T02 aceito.
