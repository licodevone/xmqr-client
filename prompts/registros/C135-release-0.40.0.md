# C135 — prerelease client 0.40.0 / bridge 0.2.0

Prompt: [C135 v1.4](../clients/135-release-0.40.0-bridge-qos2.md), versionado antes de cada ajuste.
Base `main`/C134 `130cfeb`, com [CI pós-integração](https://github.com/licodevone/xmqr-client/actions/runs/39159131168) PASS nos quatro jobs.
Estado: [prerelease v0.40.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.40.0)
publicada da fonte/tag anotada `501cde2ffbae6985f033ed8be60d9ad6dd8c18c4`.

## PASS local

- WSL Ubuntu 26.04: `cargo fmt --check`, `cargo test --locked`, Clippy em debug e release com `-D warnings`, build dos dois binários.
- Versões: `mqtt-client 0.40.0`; `mqtt-bridge 0.2.0 (xmqr-client 0.40.0)` em debug e release.
- `.deb` local `xmqr-client_0.40.0-1_amd64.deb` construído, `Version: 0.40.0-1`, arquitetura `amd64`, unit da bridge presente.
- Ciclo descartável em container Ubuntu 26.04: 0.39→0.40→0.39→0.40, configuração do operador preservada e unit não ativada: `C135_PACKAGE_LIFECYCLE_PASS`.
- Binários **extraídos do .deb local**: fixture QoS 2 TLS/mTLS de dois XMQR com `--skip-crash-checkpoint` e interoperabilidade XMQR→Mosquitto/Mosquitto→XMQR PASS. A CI de debug C134 cobre os checkpoints omitidos no build release.

## PASS da fonte exata e distribuição

- [CI 39161032551](https://github.com/licodevone/xmqr-client/actions/runs/39161032551): cinco jobs PASS — Rust/C129, pacote Ubuntu, integração da bridge, imagem e integração dos binários extraídos do pacote da CI.
- C129 `batch32`: após revisão do prazo apenas desse cenário, 21/21 cenários PASS no WSL e job Rust PASS na CI.
- QoS 2 XMQR com dois checkpoints de crash PASS no WSL e no job de integração. O oráculo de retomada acompanha `sensors/crash`, sem confundir obrigações retained independentes com falha da mensagem de teste.
- `.deb` **da CI** baixado e verificado: SHA-256 `5f453fbc17e0fbc919f457877176dd3149bf041f4aca565f2d9028d1f07a0fe6`. Binários extraídos passaram dois XMQR QoS 2 e XMQR↔Mosquitto nos dois sentidos, com checkpoints de debug pulados explicitamente; lifecycle descartável 0.39→0.40→0.39→0.40 PASS.
- Imagem `lepinheiro100/xmqr-client:0.40.0` não root, labels de versão/revisão e dois binários conferidos. Push numerado; digest remoto `sha256:1eca2f2bbebceeef84be3453943198e0f52d1017deddc24f7065c3302ac93d78` para índice OCI com variante Linux/amd64.
- Seis anexos baixados do rascunho GitHub e comparados com os arquivos locais: 6/6 SHA-256 iguais; três checksums de conteúdo PASS. `.deb` acima; tar de imagem `71297a4d08c7ea92a5c65e7e9737980eb70df2b1245c3e80e13f846ad11b04d7`; manifesto `8e3c0ed642d181ce2e8dab61ec3bdc6af57e06ba32f81a0f1a7f5e8d0a1f8de7`. Publicada como prerelease, sem mover `latest`.

## FAIL e BLOCKED

- O primeiro build release terminou com dois avisos de código usado apenas em debug (`Outgoing` e `pending_topic`). Os dois itens receberam `cfg(debug_assertions)`; Clippy release e build posteriores passaram sem esses avisos.
- `dpkg-shlibdeps` emitiu aviso sobre diversions de `/lib64/ld-linux-x86-64.so.2` da libc6 do ambiente WSL; o pacote foi construído e passou no ciclo descartável. Não é prova de build bit a bit reproduzível.
- [CI 39160243798](https://github.com/licodevone/xmqr-client/actions/runs/39160243798) da candidata `c3155d3`: quatro jobs PASS, Rust FAIL em `batch32` por EOF sob prazo de 2 s. O caso passou oito vezes no WSL; prompt revisto antes de ajustar apenas o prazo do fixture.
- [CI 39160643487](https://github.com/licodevone/xmqr-client/actions/runs/39160643487) da candidata `10514e4`: Rust, pacote, imagem e integração do pacote PASS; integração da bridge FAIL ao exigir que todo o journal ficasse vazio após crash, embora retained independente pudesse estar pendente. Prompt revisto antes de restringir o oráculo à entrada de `sensors/crash`. Uma execução local já passava; não se transforma a CI com FAIL em PASS retroativo.
- CI de commits somente de revisão do prompt foi cancelada quando superada por nova fonte. Isso não é aprovação de teste.

## Limites

0.40.0 não conclui fases 6/8 nem oferece transação exatamente uma vez entre brokers. O downgrade 0.39 preserva arquivos privados, mas a bridge 0.1.0 dessa versão não entende QoS 2/journal C134; instâncias QoS 2 devem estar paradas durante rollback. O `.deb` local inicial diferiu do `.deb` da CI; a release usa o segundo e não alega build bit a bit reproduzível.
