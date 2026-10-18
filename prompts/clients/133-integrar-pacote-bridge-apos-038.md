# C133 v1 — integrar o pacote da bridge após o client 0.38.0

## Entrada

Base imutável `v0.38.0` (`fca6fc8`), prerelease C131 publicada. A implementação
C128 do pacote da bridge está em `30791a3` e o gate de upgrade/rollback em
`24a65ba`, ambos no outro ramo `main` e ainda sem release da bridge. O client
0.38.0 distribui o journal inbound C129; a próxima versão deve preservar esse
contrato e não retroceder para 0.37.0. Antes de integrar, confirmar o resultado
das CIs exatas C128 e manter os FAILs que houver registrados.

## Objetivo e escopo

Integrar a distribuição experimental de `mqtt-bridge` ao client 0.38.0 em um
candidato 0.39.0, sem alterar a semântica C125–C127 nem a persistência C129.
Levar para o novo ramo somente o empacotamento C128: `.deb`, unit systemd
opt-in, imagem OCI, versões, documentação, fixture de pacote e gates de CI.
Resolver conflitos em `Cargo.toml`, `Cargo.lock`, `Dockerfile`, README,
VERSIONING, scripts de pacote/fixture e workflow; manter os gates C129/C131.

## Requisitos

- `mqtt-client --version` e `mqtt-bridge --version` identificam a versão
  integrada; o `.deb` e a imagem contêm ambos os binários.
- Unit da bridge instalada, validada e desabilitada por padrão; nenhuma
  configuração secreta embutida. Preservar configuração local em upgrade e
  downgrade descartáveis 0.38.0→0.39.0→0.38.0.
- O journal C129 preserva compatibilidade de leitura/rollback documentada.
  O pacote 0.38.0 não contém bridge, portanto o downgrade remove o binário e
  a unit; estado privado da bridge e do client fica sob controle do operador.
- Fixture TLS/mTLS real de dois brokers com os binários do pacote; regressões
  C129, QoS 0/1, topologia e negativos pertinentes.
- CI da fonte exata, imagem por digest, anexos com hashes remotos, tag e
  prerelease somente após todos os gates de distribuição passarem.

## Aceite e validação

Rust `fmt`, `test --locked`, `clippy -D warnings`, `build --locked`; fixtures
de bridge e journal; `build_deb.py`, extração/instalação Ubuntu 26.04 e
upgrade/rollback descartável; imagem OCI e versões dos dois binários;
`release_assets.py` e checksums. Registrar PASS, FAIL e BLOCKED separadamente
em `prompts/registros/C133-bridge-package-039.md`, com commit/CI/digests
exatos. Sem marcar fase 8/T02 ou fase 6 completa por esta distribuição.
