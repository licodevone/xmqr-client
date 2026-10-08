# Estado atual: C28 rev.2.0 executada / candidato v0.3.0

Revisão1.0 abaixo é planejamento histórico preservado. A autorização, estado real
e contrato de execução estão na revisão2.0 mais adiante; resultados em
../registros/C28-payload-binario.md. Nenhuma tag/release nova criada nesta tarefa.

# C28 - Payload binario por arquivo/stdin, revisao1.0, planejamento

Data2026-10-08. Autorizacao geral: evolucao incremental inspirada no Mosquitto.
Estado de entrada esperado: C26 implementado localmente; C27 checkpoint de
contrato/reconstrucao isolada. Manifest0.7.0 herdado sem release, HEAD26c1019,
taglocalv0.1.0 preservada; reconferir Git/local antes de EXECUTAR este prompt.
Este arquivo prepara criterios ANTES de qualquer codigo C28. Status PLANNED:
nenhum recurso abaixo existe ainda na CLI atual; nao executar nesta fatia C27.

Objetivo proximo: pub com bytes exatos sem converter UTF-8; manter --message TEXTO
compativel. Proposta --message-file PATH ou --stdin (flag sem valor), mutuamente
exclusivos com --message, exatamente uma fonte em pub; todos proibidos em sub.
Nao acrescentar aliases Mosquitto/perfis/multitopicos/timestamps nesta fatia.

Escopo previsto: novo src/payload.rs, cli/main/session ajustados, tests de bytes,
README, C27 contrato atualizado apos implementacao, matriz e registroC28.
Will permanece texto, JSONL schemaC26 preservado, fonte carregada uma unica vez
antes da rede, publish enfileirado somente uma vez, sem replay apos erro.
Limite4096bytes para TODAS fontes, vazio valido, sem remover LF/CR/BOM/NUL.
Ler no maximo4097 para detectar excedente, nao carregar arquivo inteiro para
verificar tamanho. Dispositivo/FIFO/diretorio nao sao arquivos payload regulares.
Nao imprimir conteudo/caminho sensivel nos erros; stderr operacoes, stdoutJSONL
somente conclusao de protocolo. QoS/retain/TLS/SAN/ID/retriesC26 preservados.

Projeto de execucao a resolver/documentar antes do codigo: leitura de stdin deve
ser assincrona/cancelavel com limite de tempo proposto8s ate EOF. Arquivo regular
validado pelo handle e leitura limitada; avaliar abertura que bloqueia/FIFO e
corrida de troca do path sem esconder risco. Preferir recursos Tokio fs/io-std
se necessarios, preservando versoes/lock e sem nova biblioteca instalada.
Autenticacao+stdin: senha oculta deve vir de terminal separado via rpassword
ou password-file seguroLinux0600, nunca consumir payload como senha. Ausencia
de terminal/controlador falha segura sem conectar; testar combinacoes suportadas.
Windows password-file continua proibido, nao criar bypass0600.

Criterios e testes antes da implementacao:
- CLI fontes ausente/repetida/combinada, sub rejeita, --stdin sem valor.
- Bytes0..255, NUL/invalidUTF8, LF/CRLF/BOM preservados byteabyte no wire.
- 0/4096bytes aceitos,4097 rejeitado com leitura limitada e nenhuma conexao.
- Arquivo inexistente/permission denied/diretorio/device/FIFO rejeitado seguro;
  stdinsemEOF limita tempo e Ctrl+C nao abre rede. Fixtures portas efemeras.
- --message anterior/Will/JSON/texto unchanged, retaintrue vazio correto.
- pubQoS0/1/2 terminal correto, sem reenviar arquivo/stdin depois de queda.
- Testes credenciais+stdin sem segredo ou payload misturado; limites1024topico,
  Linux0600 e TLS/SAN preservados. Unix ausente=BLOCKED, nao substituir por PASS.
- fmt --check, test --locked, clippy --all-targets -Dwarnings, build e inventario.

Aceite: registro PASS/FAIL/BLOCKED separado, documentoC27 atualizado so para
recursos realmente implementados. Nenhuma versao/tag/publicacao automatica.
Checkpoint C27 entregue ao usuario antes da execucao C28; nao afirmar que o
planejamento representa funcionalidade presente ou reprodutibilidade integral.

## Revisao2.0 - EXECUCAO C28 autorizada / proximo marco v0.3.0

2026-10-08, ANTES do codigo. Usuario esclareceu tag cliente0.2 e pede0.3 com
arquivo/stdin/binario e mensagemporlinha. Inspecao: HEAD6d25b33, tag local/remota
v0.2.0 (anotada1916ea19, commit6d25b33), sem tag literalv.0.2.0 nemv0.3.0.
v0.1.0/f8999e6 preservada. gitdiffv0.2.0srcCargo vazio: C26reconnectJSONL e
C27bootstrap JA estao no marco0.2. Tag0.2 ainda carrega manifest0.7herdado:
corrigir daqui para frente, sem alterar historia/tag. Sem publicarcrates/Git.
Preservar diffs docs/registroC26/provenance e registroC27naorastreado.

Escopo EXECUTAVEL agora: src/payload.rs novo; cli/main/session; Cargo.toml e
Cargo.lock somente versao do pacote0.3.0; --version sozinho imprime mqtt-client
0.3.0 e sai0 sem credenciais/rede. README, novo CHANGELOG.md, indice/matriz,
contratoC27com suplementoC28 e registroC28. Tag SUGERIDA v0.3.0 no checkpoint,
NAO executarcommit/push/tag/release nesta tarefa. Nenhum brokerfonte editado.

CLI: exatamente uma fonte --message TEXTO | --message-file PATH | --stdin
(flagsemvalor). --line-mode (flagsemvalor) somente com arquivo ou stdin, pub
somente. Nao adicionar aliases/perfis/multitopicos/timestamps/novodependencies.
Modo raw:0..4096bytes, sem remover LF/CR/BOM/NUL/invalidUTF8, um PUBLISHaposEOF.
Modo linhas: cada LF delimita um PUBLISH, removeLF e CR imediatamente anterior
(CRLF); CRisolado/BOM/NUL/bytesinvalidos preservados. Linhasvaziaspublicamvazio;
EOFtermina ultima linha semLF; inputtotalvazio gera ZEROpublicacoes e nenhuma
conexao. Nao criar mensagemfantasma aposLFfinal. Limite4096PORpayload apos
framing, max4099byteslidos porlinha para detectar excessocomCRLF. Arquivo/stream
pode ter mais4096total quando linhas, memoria limitada e backpressure.

Antes de rede, obter primeiro payload valido; inputerro/timeout inicial exit2;
Ctrl+Centrada exit0. Fonte regularfile checada por pathmetadata e handlemetadata,
recusar devices/FIFOs/diretorios; abertura/leituras dentro worker nativo proprio
para limitar espera mesmo em corrida de path. Erros nao incluem path/dados.
InspecaoTokio1.53.1stdin: readbloqueante nao cancelavel no pool e shutdownpode
travar. Usar UMstdthread dedicado porfontefile/stdin e mpsc boundedcapacity1,
sem tokio::stdin/spawn_blocking, semunsafe oudepsnew. LeituraOS naopode ser
interrompida portavelmente, mas espera async e cancelavel e o processo termina
sem juntar workerbloqueado. Umworker + bufferslimitados, read-ahead limitado.
Deadline8s para primeiro/PROXIMO payload, nao renovado porMQTTping/eventos.

Senha oculta rpassword usa /dev/tty/console separado, naostdinpayload; read first
payload naoconsome password. Linuxpasswordfileowner0600 unchanged. Semterminal
senha falha segura; testes secure+stdin usam arquivo0600 emLinuxnative/tmp.
Nenhum payload/secret/path sensivel em logs. TLS/SAN/Will1024/QoS preserved.

Publicar serial: uma mensagemporvez, aguardar terminal QoSantesproxima. Mesmo
AsyncClient/EventLoop/ID; manter pollingkeepalive durante esperaentrelinhas.
--reconnect-attempts somente antes PRIMEIROPUBLISH. Durante/envioouentrelinhas,
perda de rede encerra lote parcial SEMreplay e SEMreconnect; inputja pode ter
read-ahead limitado, nao reenviarlinhas. Erroaposlinhas anteriores acked NAO
reverte publicacoes confirmadas. QoS0=envio,ACKnaonegocio; QoS2recepcaoC26limite
inalterado. AceitarPingResp emawaitpub para streams maislongos, deadlinesmantidos.

JSONL schemaC26 unchanged: publish_complete umobjeto porlinha confirmada
(no modo linhas), stdout soJSON; ordem dosobjetoscorrespondeordemdeenvio. Modo
singletext preserva confirmacaodepoisDISCONNECT. Erros/statusstderr. Retain
valeparacadaPUBLISH; linhavazia+retaintrue podeapagarretained, documentar.
Cancelamento externo inclui awaitinput/poll/ACK, DISCONNECTlive max1s e descarta
filapayload/app; sem reconnectnaosaida. Runtime naoespera readOS workerbloqueado.

Aceite planejado ANTES:
- fontesausente/combinadas/repetidas/subincompativeis/line-modeinvalido/version;
- rawbytes0..255,NUL,invalidUTF8,CRLF/BOM exatos;0/4096aceitos4097semconexao;
- lineLF/CRLF,linhavazia,EOFsemLF,inputvaziosemrede;4096payload+CRLFaceito,
  4097rejeitado; reader limitado4099 e canal1/backpressure;
- arquivo nonexistent/dir/FIFO/device/permissiondenied semleitura/rede/echo;
- stdin mantido aberto8s sai2 semhangruntime; SIGINTstdin bloqueado sai0Linux,
  cancellationunitWindows e childEOF/timeoutWin; sem prometer CtrlCWin injetado;
- wire QoS0/1/2 binario/retainlinha, ACKIDssequenciais, sempublicacaosemACK,
  queda antesACK semreplay, quedaentrelinhas encerra parcial, inputerro aposACK
  exibe apenas sucessosjaobservados; brokerfrozenemtestesportas/stateproprios;
- securestdinpassword0600/TLSpeer semdadoscomosegredo; rpasswordnaostdin;
- fmt,test --locked,clippyalltargets-Dwarnings,buildWin/Linux,inventario;
  MSRV1.88naoinstalado continuaBLOCKED, naoinstalarsoftware/Mosquitto.

C27reconstrucao fica snapshotdo contrato0.2, nao alterar fontescongelados nem
usar hashespreC28 para afirmar igualdadeaposC28. ContratoC27texto recebe somente
suplementopublicoC28; roteiro de reconstruir0.3independentemente sera gatefuturo,
nao mascarar copiacomoensaionovo. RegistroC28PASS/FAIL/BLOCKED/tagexplicitafinal.
Parar antes de outros recursos ao concluir marco0.3naopublicado.
