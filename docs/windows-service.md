# Servico Windows - C151 (experimental)

O adaptador SCM opt-in usa `packaging/windows-service/Cargo.toml`, com a feature
`windows-service-runtime`. O manifest principal continua independente do SCM.
O build normal Windows agora aceita `--password-file` com validacao nativa; Linux
mantem a validacao existente de dono efetivo e modo 0600.

```text
mqtt-client.exe --windows-service xmqr-client-default "C:\ProgramData\XMQR\client\logs" --config "C:\ProgramData\XMQR\client\service.toml"
mqtt-client.exe --service-config "C:\ProgramData\XMQR\client\service.toml"
```

`--service-config` usa o mesmo carregamento, validacao, sink e sessao MQTT em
console, sem SCM. Ctrl+C/Ctrl+Break encerram pelo caminho normal. O marker
`--windows-service-capabilities` comprova somente a presenca do adaptador.
Nenhum desses comandos instala um servico.

## Configuracao

```toml
version = 1
arguments = ['sub', '--tls-password', '--host', '127.0.0.1', '--port', '8883', '--topic', 'acceptance/probe', '--username', 'acceptance', '--ca', 'C:\ProgramData\XMQR\client\ca.crt', '--password-file', 'C:\ProgramData\XMQR\client\secrets\mqtt.password', '--output', 'jsonl', '--qos', '1', '--count', '1']
output_file = 'C:\ProgramData\XMQR\client\output\messages.ndjson'
output_max_bytes = 8388608
```

O arquivo de configuracao tem limite de 64 KiB, 256 argumentos e schema fechado;
recusa NUL, comandos diferentes de pub/sub, `--profile` e caminhos relativos.
CA/certificado/chave, senha, entrada de publicacao e sink usam caminhos absolutos.
Perfis aninhados sao recusados para manter uma configuracao de lancamento verificavel.
Modos autenticados exigem `--password-file`; nunca ha prompt em servico.
Nenhuma senha e aceita em argumento ou no TOML.

`pub` exige `--message-file` absoluto e nao aceita `output_file`. `sub` exige
`--output jsonl` e `output_file` absoluto. A publicacao finita e `sub --count N`
encerram normalmente depois da conclusao; nao ha repeticao infinita implicita.
Open-lab continua opt-in e somente loopback; os padroes TLS/mTLS nao mudam.
Bridge e os modos duraveis Linux continuam fora do escopo Windows.

## Arquivos privados e saida limitada

Configuracao, senha e saida devem existir previamente. Seu arquivo e diretorio
pai imediato devem ter dono confiavel e DACL explicita ou herdada que conceda
acesso somente a identidade atual, SYSTEM, Administradores e/ou LocalService.
Permissoes para Everyone, Authenticated Users, Users ou outras identidades sao
recusadas, mesmo com uma ACE deny. DACL nula/vazia, dono desconhecido, ACEs de
objeto/callback e pontos reparse no arquivo ou em qualquer ancestral falham fechados.
A validacao usa o handle aberto; ancestrais ficam presos durante a abertura e o
handle do arquivo impede substituicao/remocao e outro escritor durante o uso.
As APIs seguras sao `windows-permissions = 0.2.4` e `winsafe = 0.0.29` (advapi),
fixadas nos dois locks. Nao ha bloco unsafe no codigo proprio.

O operador deve preparar DACLs antes de gravar a senha. A conta do servico precisa
ler configuracao/senha e ler/escrever a saida. A saida nao e criada automaticamente.
A senha e UTF-8 de ate 1024 bytes, aceita um LF/CRLF final e recusa linhas internas;
a leitura maxima e 1027 bytes, sem registrar o conteudo.

O sink acrescenta objetos NDJSON completos, preservando bytes binarios do payload
e metadados MQTT 5. A capacidade e de 4 KiB a 64 MiB, padrao 8 MiB. Ao atingir o
limite, o processo falha antes de confirmar a mensagem; nao trunca nem rotaciona
registros existentes. Pare o servico, arquive/consuma o arquivo e prepare um novo
arquivo privado para liberar capacidade. Um tail parcial depois de falha de IO ou
crash e recusado na reabertura, sem reparo/truncamento silencioso.

## Limites de confirmacao e ciclo de vida

QoS 1 PUBACK e QoS 2 PUBREC so sao enfileirados depois de write_all, flush e
sync_data bem-sucedidos no sink. QoS 0 nao tem ACK e pode perder mensagens durante
falha/desconexao. Filtros de exclusao/retained configurados descartam intencionalmente
as mensagens correspondentes antes do ACK. Fila de ACK cheia ou falha de disco
encerra a sessao, sem continuar consumindo em stdout invisivel.

Isso confirma persistencia local solicitada ao sistema operacional, nao consumo
por outra aplicacao, tolerancia a toda falha de hardware ou entrega exatamente
uma vez. Crash entre sync_data e ACK pode duplicar registros. Estado inbound QoS 2
nao e persistido no Windows; nao ha promessa de retomar o handshake apos restart.
MQTT 3.1.1 mantem o comportamento de reconexao existente; MQTT 5 nao ganha um
journal Windows. Retencao/limpeza de sessao do broker continuam valendo.

SCM recebe StartPending, Running, StopPending e Stopped. Running significa que
configuracao/entrada foram preparadas, nao CONNACK ou confirmacao de negocio.
STOP recebido durante inicializacao fica retido; StartPending nunca sobrescreve
StopPending. O encerramento usa o cancelamento existente da sessao. `service.log`
contem somente mensagens fixas de ciclo de vida, rotacao 1 MiB + um anterior.

A bandeja/instalador sao mantidos no broker. Controle SCM real e aceite de conta
de servico continuam requerendo VM Windows; nao foram instalados servicos no host.
Historico: [C150](../prompts/registros/C150-windows-service.md).
Implementacao e gates: [C151](../prompts/registros/C151-windows-private-runtime.md).
