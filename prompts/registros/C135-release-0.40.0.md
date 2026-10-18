# C135 — candidata client 0.40.0 / bridge 0.2.0

Prompt: [C135 v1.2](../clients/135-release-0.40.0-bridge-qos2.md), versionado antes dos ajustes.
Base `main`/C134 `130cfeb`, com [CI pós-integração](https://github.com/licodevone/xmqr-client/actions/runs/39159131168) PASS nos quatro jobs.
Estado: candidata local; CI da revisão exata, tag, imagem remota e prerelease pendentes.

## PASS local

- WSL Ubuntu 26.04: `cargo fmt --check`, `cargo test --locked`, Clippy em debug e release com `-D warnings`, build dos dois binários.
- Versões: `mqtt-client 0.40.0`; `mqtt-bridge 0.2.0 (xmqr-client 0.40.0)` em debug e release.
- `.deb` local `xmqr-client_0.40.0-1_amd64.deb` construído, `Version: 0.40.0-1`, arquitetura `amd64`, unit da bridge presente.
- Ciclo descartável em container Ubuntu 26.04: 0.39→0.40→0.39→0.40, configuração do operador preservada e unit não ativada: `C135_PACKAGE_LIFECYCLE_PASS`.
- Binários **extraídos do .deb local**: fixture QoS 2 TLS/mTLS de dois XMQR com `--skip-crash-checkpoint` e interoperabilidade XMQR→Mosquitto/Mosquitto→XMQR PASS. A CI de debug C134 cobre os checkpoints omitidos no build release.

## FAIL e BLOCKED

- O primeiro build release terminou com dois avisos de código usado apenas em debug (`Outgoing` e `pending_topic`). Os dois itens receberam `cfg(debug_assertions)`; Clippy release e build posteriores passaram sem esses avisos.
- `dpkg-shlibdeps` emitiu aviso sobre diversions de `/lib64/ld-linux-x86-64.so.2` da libc6 do ambiente WSL; o pacote foi construído e passou no ciclo descartável. Não é prova de build bit a bit reproduzível.
- CI da revisão final, pacote da CI, digest remoto, anexos e tag: PENDING. Não publicar nem marcar distribuição aceita antes desses gates.

## Limites

0.40.0 não conclui fases 6/8 nem oferece transação exatamente uma vez entre brokers. O downgrade 0.39 preserva arquivos privados, mas a bridge 0.1.0 dessa versão não entende QoS 2/journal C134; instâncias QoS 2 devem estar paradas durante rollback.
