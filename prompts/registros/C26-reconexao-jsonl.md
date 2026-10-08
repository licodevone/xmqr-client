# Registro C26 - reconexao limitada e JSON Lines, 2026-10-08

Prompt: clients/26-reconexao-jsonl.md v1.0 escrito e persistido antes do codigo;
rev1.1 antes do alinhamento de indice/matriz; rev1.2 antes da protecao QoS2.
Autorizacao: primeiro marco de melhorias delegado, seguido de alinhamento de prompts.
Plataforma: Windows/computador lico, D:/projects/my-project/xmqr-client.
Entrada/final: HEAD26c10191c9ec0b6d3181488794584bfbf24ed756, origin
https://github.com/licodevone/xmqr-client.git; entrada limpa. Manifest0.7.0
herdado/local, publish=false; taglocalv0.1.0 em f8999e6e8c7593fbcbbccb9657089a75cc10985d
preservada. Versao propria/publicacao nao decidida; nenhum commit/push/tag/release.
Toolchain cargo/rustc1.98.1; MSRV declarado1.88 nao comprovado pelo gate atual.

## Implementacao

CLI: --reconnect-attempts0..10 default0 e --output text|jsonl defaulttext.
Orcamento total por processo sem reset; backoff100/200/400/800/1600/3200ms,
cap5s, mesmo EventLoop e Client ID. Recupera sub, reassina somente se sessao
nao retomada ou SUBACK anterior nao confirmado. Auth/TLS/protocolo sem retry.
Pub pode retentar conexao inicial, mas nunca reenvia PUBLISH interrompido:
falha com resultado desconhecido. JSON stdout somente objetos, payload_bytes
sem perda; logs/erros stderr, sem credenciais. Texto anterior preservado.
Cancelamento cobre CONNECT/backoff/ACK/idle, descarta fila e tenta DISCONNECT
live por ate1s, sem reconectar; unavailable fecha. Quotas/TLS/Will preservados.

Inspecao real da dependencia rumqttc0.25.1 em registry/src: eventloop.rs poll
reconecta; clean move pending; CONNACK sem session_present limpa pending.
state.rs clean apaga incoming_pub QoS2; handle_incoming_pubrel rejeita ID
que nao esteja ali. Assim uma recepcao QoS2 interrompida durante handshake
falha explicitamente sem retentar; deadline fixo8s para concluir QoS2 recebido.
Essa limitacao foi registrada antes do ajuste e testada, sem substituir codec.

Arquivos: src/cli.rs, src/main.rs, src/session.rs, novo src/output.rs,
tests/wire.rs, README.md, prompts/clients/26-reconexao-jsonl.md,
prompts/README.md, prompts/COBERTURA.md, este registro; C19-C24 receberam apenas
aviso local de historico/caminhos da origem. Cargo.toml/lock inalterados.
Nenhum fonte/processo/estado do broker vizinho editado/executado.

## PASS

- cargo fmt --all -- --check: exit0.
- cargo test --locked --offline: 28 testes, 19 unitarios + 9 fixtures wire,
  zero falhas finais. Fixtures portas efemeras127.0.0.1, accept/read/process
  com deadlines3..7s, sem Mosquitto, certificados reais ou estado duravel.
- CLI: defaults/limites/repeticao/negativos, sessao/QoS/Will/1024bytes preservados.
- JSON: payload vazio/binario0,255,controles/Unicode, escaping e texto compatvel;
  stdout wire exato, pub0/1/2 e stderr operacional.
- Wire: ID/CONNECT estavel clean=false; sessao presente sem reassinar e ausente
  reassinada; contagem mantida; 3 conexoes max para2retries; EOF inicial/default0;
  auth/NotConnAck sem retry e sem vazar payload do peer; pub1/2 interrompida sem
  replay; pub0/1/2 ACK terminal correto; sub1/2 handshake antes de count/DISCONNECT;
  publicacao antes de SUBACK limitada por count; queda apos PUBREC2 sem retry.
- Cancelamento: DISCONNECT live sem escoar PUBLISH enfileirado e cancelamento
  da espera progressiva testados deterministicamente, sem injetar Ctrl+C real.
- cargo clippy --locked --offline --all-targets -- -D warnings: exit0.
- cargo build --locked --offline: exit0. Linker informa criacao de .lib/.exp,
  aviso informativo da toolchain Windows; Clippy -Dwarnings nao teve warnings.
- Inventario: CARGO_NET_OFFLINE=true, python bundled -B -Xutf8
  scripts/license_inventory.py --check: Project MIT declaration and dependency
  license inventory match. Sem instalacoes/upgrades/dependencias novas.
- git diff --check: exit0; manifest/lock e TLS/credentials sem diff.

## FAIL (intermediarios resolvidos)

- Primeira rodada fixtures5/6 falharam: socket aceito herdou nonblocking no
  Windows. Corrigido set_nonblocking(false); todas fixtures finais PASS.
- Clippy apontou semicolon e match arms iguais; corrigidos, gate final PASS.
- Primeiro teste de cancelamento dependia de recusa TCP em100ms no Windows;
  substituido por teste da futura real de backoff, sem acesso a rede, PASS.
- Script temporario de edicao teve erro de sintaxe (backticks docs), corrigido;
  nao houve alteracao de fonte nessa tentativa. Nenhuma falha final pendente.

## BLOCKED / nao comprovado

- WSL --status exit0 indica Ubuntu-24.04/WSL2, mas executar /bin/true excedeu
  deadline10s; somente o processo wsl lancado pela verificacao foi encerrado.
  Gates Linux/0600 e integrações Unix nao executados. Nao alegar WSL funcional.
- Integracao real TLS/SAN, retained armazenado, Will, restart/takeover/ACL com
  broker independente nao executada: broker vizinho sob outra tarefa. Requer
  coordenacao e --client explicito; nenhum estado real foi usado.
- Mosquitto externo nao instalado/executado. MSRV1.88 exato nao executado.
- Reconstrucao do zero somente com prompts nao executada e ainda GAP. C19-C24
  historicos mencionam contratos/paths ausentes; C25 depende de fonte/backup.
  Indice/matriz explicitam isso; proposta C27 bootstrap independente antes
  de prometer reproducibilidade. Nao houve bootstrap paralelo.

Marco local concluido nos gates Windows. Parar nesta fatia para usuario decidir
versao propria e publicacao antes de prosseguir outros recursos. Nao anunciar release.

Verificacao final adicional: links locais ativos README/prompts/README/prompts/COBERTURA PASS; referencias quebradas historicas explicitadas, nao tratadas como bootstrap. rustup nao lista toolchain1.88; sem instalacao. Git HEAD/tag/manifest/lock preservados.
