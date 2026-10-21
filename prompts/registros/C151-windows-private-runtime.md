# C151 - Runtime privado Windows (v1.1)

Data: 2026-10-21. Continuidade de C150. Trabalho existente, incluindo
`.serena/project.yml`, prompts e manifest experimental C150, preservado.
Sem commit, push, instalacao de servico, MSI no host ou alteracao global de ACL.

## Entrega

- Senha Windows: arquivo existente, UTF-8 limitado, DACL/dono validados no handle.
- Configuracao/senha/sink requerem parent privado; reparse em qualquer ancestral,
  ADS/device aliases, UNC, nomes DOS reservados e caminhos relativos recusados.
- Dependencias Windows exatas: windows-permissions 0.2.4 + winsafe 0.0.29 (advapi).
  API `wrappers::GetSecurityInfo(...SE_FILE_OBJECT...)`: a conveniencia
  WindowsSecure usa tipo desconhecido e nao serve para arquivos (erro 87).
- `--service-config ABS_CONFIG` e SCM usam a mesma preparacao e `run_cli` com
  parse unico. Nao ha senha inline, prompt em servico ou perfil aninhado.
- Sub JSONL para arquivo existente, privado, escritor exclusivo; limite
  configuravel 4 KiB..64 MiB (8 MiB padrao), sem truncamento/rotacao automatica.
- `write_all` + `flush` + `sync_data` antes de enfileirar PUBACK/PUBREC.
  Falha de disco/capacidade encerra sem ACK; tail parcial impede reabertura.
- SCM sincronizado com broker: STOP retido na inicializacao, primeiro estado
  serializado com callback, Ctrl+Break no console do runtime SCM.
- Linux conserva dono efetivo/0600 e comportamento de leitura; bridge nao ganha
  suporte Windows. Teste WSS/outbox Linux tem cfg Linux e equivalente nao-Linux
  exige a recusa explicita; runtime de outbox nao foi alterado.

## Comandos e gates

PowerShell no repositorio client; ambiente dos comandos nativos:

```powershell
$env:CARGO_HOME='C:\Users\licod\AppData\Local\Temp\xc151'
$env:TMP=(Resolve-Path target/c151-temp).Path
$env:TEMP=$env:TMP
$env:CARGO_BUILD_JOBS='2'
```

| Comando | Resultado |
| --- | --- |
| cargo check --manifest-path packaging/windows-service/Cargo.toml --target x86_64-pc-windows-msvc --bin mqtt-client --locked --offline --target-dir ../xmqr/target/p239-service-proof | PASS |
| cargo build --manifest-path packaging/windows-service/Cargo.toml --target x86_64-pc-windows-msvc --bin mqtt-client --locked --offline --target-dir ../xmqr/target/p239-service-proof | PASS |
| cargo test --manifest-path packaging/windows-service/Cargo.toml --target x86_64-pc-windows-msvc --bin mqtt-client --locked --offline --target-dir ../xmqr/target/p239-service-proof | PASS 95/95, zero ignorados |
| cargo clippy --manifest-path packaging/windows-service/Cargo.toml --target x86_64-pc-windows-msvc --all-targets --locked --offline --target-dir ../xmqr/target/p239-service-proof -- -D warnings | PASS |
| --windows-service-capabilities / --version | PASS schema=1 component=client service_entrypoint=true / 0.43.0 |
| cargo build --manifest-path Cargo.toml --target x86_64-pc-windows-msvc --bins --locked --offline --target-dir target/c151-native | PASS; aviso preexistente unused_mut bridge_multi.rs:213 |
| cargo test --manifest-path Cargo.toml --target x86_64-pc-windows-msvc --bin mqtt-client --locked --offline --target-dir target/c151-native | PASS 93/93, zero ignorados |
| cargo clippy --manifest-path Cargo.toml --target x86_64-pc-windows-msvc --bin mqtt-client --locked --offline --target-dir target/c151-native -- -D warnings | PASS |
| cargo fmt --all -- --check e cargo fmt --manifest-path packaging/windows-service/Cargo.toml -- --check | PASS |
| git diff --check | PASS |

Binario debug usado na integracao coordenada:
`../xmqr/target/p239-service-proof/x86_64-pc-windows-msvc/debug/mqtt-client.exe`.
SHA256 `95a130d057cbc77c5daee04909a850d18672babd5ff12b7fe74840439b302c58`.
Depois desse build, alteracoes de fonte foram apenas testes/formatacao/lints sem
mudanca intencional de comportamento. O coordenador produz o binario release final.

O build normal preserva a separacao SCM: --windows-service-capabilities retorna
saida 2, --version retorna 0.43.0. mqtt-bridge --version retorna 0.5.0.
Binarios normais: `target/c151-native/x86_64-pc-windows-msvc/debug/mqtt-client.exe`
e `mqtt-bridge.exe` no mesmo diretorio.

## Evidencia nativa e historico de falhas

Testes reais de ACL recusam concessoes a Everyone, Authenticated Users e Users
no arquivo e parent; positivo privado, senha CRLF, tamanho/UTF-8/linhas invalidas,
limite/tail parcial do sink e configuracao nao interativa passam. Descritores
nulos/vazios, deny mascarando concessao ampla e dono desconhecido sao recusados.
O teste de reparse le o junction real `C:\Documents and Settings`, verifica o
atributo e exige a mensagem exata do validador. Nunca modifica esse caminho.
`XMQR_TEST_REPARSE_DIRECTORY` pode fornecer outro junction real em CI.

Criar junction de fixture foi negado pelo sandbox, tanto cmd como PowerShell;
por isso o teste usa o junction existente, sem ignorar o gate. A preparacao de
fixtures usou ACEs herdaveis OICI para que aplicar DACL privada no parent nao
remova acesso a arquivos filhos. As primeiras execucoes falharam nesses pontos
e no teste Linux/outbox sem cfg; todas as causas foram corrigidas/documentadas.

Cache inicial em Temp perdeu Cargo.toml/.crate e causou erros de resolucao;
compilacao AWS-LC com TMP herdado tambem falhou. O coordenador restaurou o cache
privado por checksums e atualizou timestamps, e TMP/TEMP passaram a ser locais.
Avisos de hardlink incremental do ambiente foram tratados pelo fallback de copia
do Cargo; Clippy estrito nao apresentou aviso de codigo proprio.

O coordenador comprovou nove cenarios debug: mTLS MQTT 3.1.1/5 QoS 1/2 com
subscriber --service-config gravando JSONL privado, senha/CA/SAN incorretos
recusados e operacao admin autenticada/restart. Evidencia agregada e gate final
release/limite/parada ficam no registro P242 do broker.

## Limites

SCM Start/Stop real e instalacao MSI continuam NOT_RUN no host e exigem VM.
QoS 0 nao tem ACK. ACK MQTT nao prova consumo externo; crash apos sync_data e
antes do ACK pode duplicar saida. Nao ha journal inbound QoS 2 Windows nem
promessa de exatamente uma vez/retomada de handshake depois de restart.
Capacidade cheia exige parar, consumir/arquivar e preparar novo arquivo privado.

Documentacao: [operacao](../../docs/windows-service.md),
[prompt C151](../clients/151-windows-private-runtime.md).
