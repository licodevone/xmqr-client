# C58 rev1 — assinatura MQTT 5 inicial

Data: 2026-10-16. Entrada: `main` em `0f10478`, client 0.13.0, C57 aceito e CI aprovada. Prompt anterior ao código.

## Objetivo e escopo

Permitir `sub --protocol-version 5` com filtros existentes, QoS0/1, Clean Start, transporte/credenciais/perfis atuais, `--count` e filtros locais de retained/exclusão. Aguardar SUBACK MQTT 5 com reason code por filtro antes de declarar assinatura ativa; emitir publicações nas saídas existentes e confirmar QoS1 recebido. Encerrar com DISCONNECT ao atingir count; Ctrl+C mantém encerramento seguro.

Arquivos: `src/{cli,v5_session,output}.rs`, testes de CLI/wire/integração, README, changelog e registro C58. Sem dependências ou migração.

## Limites e aceite

Recusar QoS2, Will, sessão persistente, reconexão e opções não suportadas antes da rede. Não tratar SUBACK negativo como sucesso; timeout e falha de rede são erro, sem inventar entrega. Preservar default MQTT3.1.1, TLS/SAN, limites de pacote/tópico/payload e não expor senha. Testar CLI, SUBACK positivo/negativo, PUBLISH/retained e par real com broker P116, além de fmt, test, Clippy, build e CI. Registrar PASS/FAIL/BLOCKED reais antes de tag.

Norma: <https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html>, §§3.8–3.9.

Rev2: o primeiro teste wire falhou porque `set_max_packet_size` adicionou propriedade CONNECT 0x27, ainda recusada pelo broker P115. Manter CONNECT sem essa propriedade e validar o tamanho de cada PUBLISH recebido após decodificação, preservando o contrato do subconjunto. Repetir os testes wire antes do aceite.

Rev3: revisão normativa OASIS §3.8.4 permite PUBLISH antes de SUBACK. Aceitar essas mensagens sem declarar assinatura ativa; `--count` só encerra após SUBACK validado. Após atingir count, dirigir o event loop até a saída de DISCONNECT, permitindo o ACK QoS1 pendente.

Rev4: o novo teste wire parou por erro do vetor: Remaining Length 9 foi declarado para corpo de 8 bytes. Corrigir o fixture para 8 e repetir, sem alterar o código de produção por esse sintoma.

Rev5: o negativo SUBACK fechava o socket do servidor antes do processamento pelo client e reportava apenas EOF. O fixture agora espera o encerramento pelo client, permitindo verificar a recusa real.
