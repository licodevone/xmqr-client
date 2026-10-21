# C150 - serviço Windows experimental

- Prompt: [C150 v1.2](../clients/150-windows-service.md), preparado antes do código;
  v1.2 isolou a dependência após falha real de resolução offline.
- Pedido: serviço controlável pela bandeja, execução paralela autorizada.
- Data: 2026-10-20, Windows nativo, Rust/Cargo 1.99.1, PowerShell.
- Entrada: fbbf7af768493c7c6adfdcef06750d6402a7a658 / 0.43.0;
  alteração preexistente `.serena/project.yml` preservada.

## Implementação

CLI retorna código ao chamador; modo serviço encaminha stop/shutdown ao sinal
existente e reporta falha ao SCM. Config dedicado limitado a 64 KiB, comandos
pub/sub e até 256 argumentos. Publicação por arquivo absoluto aceita; stdin,
payload inline, senha interativa e assinatura em serviço recusados antes da
rede. Sub não pode descartar dados em stdout de serviço. Guarda Linux/WSL de
senha privada, outbox e bridge preservada. Running não prova conexão MQTT.

Fontes alteradas: main.rs/signals.rs, service_config.rs e windows_service.rs;
manifest experimental em packaging/windows-service, documentação e prompt.
Cargo.toml principal só declara check-cfg do valor experimental; Cargo.lock,
dependências normais e versão preservados. O manifest experimental compartilha
a fonte e habilita windows-service-runtime. Seu lock permanece ausente até
resolução real da dependência; não há binário de serviço aceito.

## Validação real

| Comando/checagem | Resultado |
| --- | --- |
| cargo fmt --all -- --check | PASS, saída 0 |
| cargo fmt --manifest-path packaging/windows-service/Cargo.toml -- --check | PASS, saída 0 |
| git diff --check | PASS, saída 0; avisos de CRLF preservados |
| cargo check --offline --all-targets (antes de isolar) | BLOCKED: windows-service ausente do cache, saída não zero |
| cargo test --locked --offline | FAIL: bridge 11 PASS / 4 FAIL, PersistError PermissionDenied Windows |
| cargo test --locked --offline --bin mqtt-client | 86 PASS / 2 FAIL; testes novos de configuração/recusa PASS |
| cargo build --locked --offline --bins | PASS, saída 0; aviso preexistente na bridge |
| cargo clippy --locked --offline --all-targets -- -D warnings | FAIL: unused_mut preexistente em bridge_multi.rs:213 |
| cargo clippy --locked --offline --bin mqtt-client -- -D warnings | PASS, saída 0 |
| cargo check --manifest-path packaging/windows-service/Cargo.toml --offline | BLOCKED: windows-service ausente, saída não zero |
| Smoke executável normal --version / --help | PASS, saída 0 |
| Smoke executável normal --windows-service-capabilities | PASS: recusado, não é falsamente anunciado como serviço |
| Verificação de alinhamento dos manifests por tomllib | PASS: versão/dependências base idênticas |
| CARGO_NET_OFFLINE=true python scripts/license_inventory.py --check | BLOCKED: metadata de outras plataformas requer r-efi 5.3.0 ausente |
| wsl.exe --list --quiet | BLOCKED: Wsl/E_ACCESSDENIED |

As duas falhas do binário client são teste de outbox Linux/WSL e bind de socket
negado pelo ambiente (10013). Os testes falhos estão em módulos não alterados
nesta fatia; não foram ocultados ou convertidos em sucesso. A suíte completa
abortou na bridge antes de executar o binário client, por isso o segundo comando
foi necessário. Sem execução de TLS/MQTT externo ou instalação de serviço.

O probe de capabilities do build experimental retorna JSON schema=1,
component=client, service_entrypoint=true; ele só comprova a presença do
adaptador, não prontidão/segurança MQTT. Não foi executado porque não há build
experimental validado. Categorias fixas de erro são registradas sem conteúdo
de exceções. STOP é sinalizado antes do lock de estado; callback não faz IO.
Sem commit, push, tag, release ou mudança de permissão/serviço do host.

## Comprovação P239 — 2026-10-21 (v1.3/v1.4)

Revisões preparadas antes da execução; HEAD fbbf7af e alterações anteriores
preservados. Cargo gerou packaging/windows-service/Cargo.lock de verdade,
com windows-service 0.8.1. Locks/manifests principais e vendor preservados.
O Clippy do adaptador encontrou if let aninhado no callback STOP; convertido
em let-chain sem alterar sinalização/IO/semântica. Repetição offline Windows
x64 MSVC/Rust 1.99.1: fmt, build --locked, marker real, --version, 1 teste de
status, 2 testes de service_config e Clippy --all-targets -D warnings PASS.
Config relativa recusada com saída 2; dispatcher fora do SCM recusado com
saída 1. Não houve instalação nem execução como serviço.

Runner e relatório agregado no broker:
[P239](../../../xmqr/prompts/registros/P239-windows-services-tray.md) e
[comandos/hashes](../../../xmqr/docs/results/p239-windows-service-proof-20261021.json).
Executável comprovado: ../xmqr/target/p239-service-proof/x86_64-pc-windows-msvc/debug/mqtt-client.exe.
Os bloqueios de dependência/build acima são históricos. Credenciais/saída
nativas, controle SCM e operação MQTT como serviço continuam sem aceite.
