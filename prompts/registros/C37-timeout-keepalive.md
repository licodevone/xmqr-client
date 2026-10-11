# C37 — timeout e keepalive configuráveis

Data: 2026-10-11. Prompt: C37 rev4. Entrada: HEAD 5d1f74207f953d31b9a246c8626bfacf359f1dd9, versão 0.5.0. Saída local: 0.6.0; commit/tag/push coordenados pelo agente raiz após revisão. Alteração preexistente em .serena/project.yml não foi editada por este agente.

## Implementação

CLI valida --keepalive-seconds 1..65535/default30 e --timeout-seconds 1..300/default8; valores repetidos, faltantes e inválidos rejeitados. CONNECT transmite keepalive configurado; EventLoop reutilizado mantém opções na reconexão. Timeout configurado substitui deadline de protocolo para CONNACK por poll/tentativa, SUBACK, PUBLISH/PUBACK/PUBCOMP, QoS2 recebido e DISCONNECT normal. Não muda retries, limites de bytes, TLS/SAN, credenciais, idle90s, input8s ou cancelamento1s. Manifest/lock atualizados sem mudar dependências.

## PASS — Windows x86_64-pc-windows-msvc

Rust 1.98.1 (48a229cea 2026-09-01), Cargo via ferramenta instalada. Gates finais realizados com manifest/lock 0.6.0:

- cargo fmt --all -- --check: PASS.
- cargo test --locked: PASS, 28 unitários e 17 wire; 0 falhas (wire 8.58s).
- cargo clippy --locked --all-targets -- -D warnings: PASS.
- cargo build --locked --bins: PASS.
- mqtt-client --version: mqtt-client 0.6.0.

Novos testes: defaults e extremos CLI; zero/acima do limite/não inteiro/negativo/faltante/repetido; bytes CONNECT default30/extremos1/65535; reconexão com keepalive47 e timeout2; peer silencioso sem CONNACK/SUBACK/PUBACK/PUBCOMP e QoS2 recebido sem PUBREL com timeout1, saída limitada e sem retry/replay. Fixtures existentes seguem cobrindo conclusão normal QoS0/1/2 e DISCONNECT.

Build/test emitem warning informativo linker_messages do MSVC sobre criação de biblioteca/objeto; compilação e testes retornaram sucesso. Clippy com -D warnings passou sem diagnósticos. Não foi necessário alterar implementação após falhas.

## FAIL

Nenhuma falha de código/gate nesta execução.

## BLOCKED / NOT_RUN

- BLOCKED Unix/WSL: wsl --list --quiet retornou stdout/stderr vazios, sem distribuição disponível; SIGTERM/0600 e integração Unix não executados.
- NOT_RUN MSRV Rust 1.88 exato; versão disponível é 1.98.1.
- NOT_RUN interoperabilidade externa Mosquitto/XMQR e TLS/mTLS cruzado nesta fatia. Fixtures loopback próprias não substituem essa evidência.
- NOT_RUN pacote Ubuntu/build/install/upgrade/release; nenhum novo pacote anunciado. P55 broker mantém aceite Unix pendente.

## Referências verificadas

[OASIS MQTT 3.1.1 §3.1.2.10](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html) define keepalive em segundos de 16 bits e permite zero no protocolo. Este CLI exige valor positivo por contrato C37. [rumqttc 0.25.1 MqttOptions::set_keep_alive](https://docs.rs/rumqttc/0.25.1/rumqttc/struct.MqttOptions.html#method.set_keep_alive) é a API usada; não alteramos os timeouts internos da biblioteca. Fontes consultadas em 2026-10-11.

## Checkpoint anterior preservado (histórico; não evidência atual)

### C37 — ponto de retomada

PREPARED, implementação NOT_RUN. Prompt37-timeout-keepalive.md criado antes do
código; baseclient1f22a18, versão0.5.0. Próximo: implementar opções de timeout e
keepalive, fixtures wire/negativos, gates e atualização documental. Não houve
mudança de versão ou tagC37. Usuário consultou encerramento do chat antes da implementação.

C36: commit1f22a18/taglocal c36-paridade-v1. Push falhou por credenciais Windows;
conector GitHub403. BrokerP55: commit10e1b59, sem tag/release, aceite Unix pendente.
Autorizações persistentes: implementar por prompts, commit/push de cada entrega,
tag após evolução aceita e release quando justificada. Não alegar publicação.

Serena foi instalado isoladamente em ../.mcp-runtime e handshake MCP validado
com xmqr-client ativo. Conexão é desta sessão, não configuração global corrigida.
Pencil não foi desconectado: configglobal fora da permissão de escrita, sem
controle de desconexão disponível; não usar ferramentas Pencil na retomada.

## Coordenação final rev4 — pacote e publicação

Revisão independente do diff não encontrou regressão impeditiva. Cargo test --locked --release: PASS/exit0, 28 unitários e 17 wire; cargo build --locked --release --bins: PASS/exit0. Logs em target/c37-release-tests.log. Pacote portátil target/c37-release/xmqr-client-0.6.0-windows-x86_64.zip contém executável otimizado, LICENSE e notas; extração via Expand-Archive aprovada, executável extraído idêntico ao testado (SHA-256), --version 0.6.0 e --help com opções C37: PASS/exit0.

SHA-256 ZIP: 7641da11e980e5395d469e9d255e47bcfd8273625e9aa74539cf3e3c58da20de. SHA-256 EXE: 976d610d6a401ed70741ce3f281003fae173b8d3bd0a1b6744bd5dfad9d671f6. SHA256SUMS em target/c37-release. Pacote experimental local; não é instalador/serviço nem pacote Ubuntu. Marco aceito no escopo portátil C37; fase1/broker P55 continuam abertos. Não publicar release Ubuntu sem build/install/upgrade/rollback pertinentes.

Autorização explícita do usuário permite commit, push e tag de cada evolução aceita. git ls-remote --tags origin v0.6.0 com http.sslBackend=openssl retornou exit0/vazio (tag ausente); verificação TLS preservada. git push --dry-run origin HEAD: BLOCKED/exit128, wincredman indisponível e Username GitHub inacessível na sessão. Coordenador tentará push final após commit/tag local; não presumir publicação por criar a tag. Alteração preexistente .serena/project.yml excluída do commit.
