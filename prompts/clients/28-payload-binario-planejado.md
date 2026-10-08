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
