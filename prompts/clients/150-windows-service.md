# C150 v1.2 - adaptador Windows Service

Revisão preventiva v1.1: config dedicada evita argumentos operacionais no SCM.
O escopo inicial aceita somente pub com fonte de payload em arquivo absoluto;
sub é recusado antes da conexão até existir saída privada Windows validada,
evitando ACK com descarte silencioso em stdout sem console. A leitura de senha
privada continua Linux/WSL: modo autenticado Windows falha fechado. Não criar
arquivo de payload nem reduzir ACL para contornar o bloqueio. service_config.rs
e testes de parsing/recusas fazem parte do escopo.

Entrada: 0.43.0, fbbf7af768493c7c6adfdcef06750d6402a7a658. Preservar alteração
preexistente .serena/project.yml. Pedido autorizado: controle pela bandeja XMQR.
Objetivo: modo opt-in de serviço para mqtt-client pub/sub com encerramento
cooperativo. Escopo: Cargo.toml/lock, main.rs, signals.rs, windows_service.rs,
docs e inventário de licenças. Independência do broker preservada, sem path/Git
dependency nova ao broker. Bridge Linux não ganha suporte Windows implicitamente.

Contrato comum P239: --windows-service NOME LOG_DIR --config CAMINHO_ABSOLUTO,
TOML limitado a 64 KiB com version=1 e arguments=[argumentos CLI], SCM status e parada;
nenhum prompt de senha/stdin em serviço, sem segredos em argumentos/registro.
CLI e sinais Unix preservados; Windows permanece sujeito aos gates TLS/ACL.
Sem versão nova, migração, instalação no host, commit/push/tag/release.

Aceite: fmt/test/clippy/build; parsing, recusa de entrada interativa, stop
cooperativo e CLI existente. Registrar PASS/FAIL/BLOCKED separadamente em
prompts/registros/C150-windows-service.md; serviço real exige VM/elevacão.

## Revisão v1.2 antes da correção do bloqueio de dependência

`cargo check --offline --all-targets` não encontra windows-service no cache;
a resolução online também está bloqueada por DNS no ambiente. Manter a nova
dependência no manifest principal impediria os gates --locked anteriores.
Isolar o build de serviço em packaging/windows-service/Cargo.toml, com fonte
compartilhada e feature windows-service-runtime apenas no manifest experimental.
O manifest principal e Cargo.lock mantêm as dependências anteriores. O build
experimental permanece BLOCKED até obter a dependência e gerar lock real.
Não fabricar checksum, copiar artefato antigo como novo nem declarar build PASS.

## Revisão v1.3 — comprovação integrada P239 (2026-10-21)

Pedido: resolver a pendência dos adaptadores/bandeja do registro P239 do broker.
Entrada: fbbf7af, alterações C150 e .serena/project.yml preservadas.
Gerar lock experimental real; compilar e testar o adaptador; corrigir somente
erros dessa integração, com prompt revisado antes da correção. Arquivos:
packaging/windows-service/Cargo.lock, src/windows_service.rs se necessário,
docs/windows-service.md e registro C150. Aceite: fmt, build --locked,
testes de configuração/estado, marker real, negativo fora do SCM e Clippy;
gates globais se houver mudança funcional. Evidências com saídas e hashes
agregadas pelo P239. Preservar versão, vendor, credenciais e restrição de sub;
não alterar serviços/permissões do host nem inferir controle SCM pelo build.

### v1.4 — lint do adaptador compilado

Clippy real encontrou collapsible_if no callback STOP. Usar let-chain para
os locks/handle, sem mudança de comportamento; repetir gates do manifest
experimental e preservar o resultado inicial no relatório P239.
