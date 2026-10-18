# Assinatura MQTT5 com journal de recepção

Checkpoint C129/C130,2026-10-18: implementado e testado em Linux/WSL,
**ainda aguardando aceite integrado da fase6 e distribuição**. Correções de
SubscriptionIdentifier e journal órfão foram integradas após testes próprios.
Confira o [registro com fontes/resultados](../prompts/registros/C129-subscription-inbound-state.md).

## Para que serve

Sessão persistente no broker e journal no client são coisas diferentes.
O broker conserva assinaturas e obrigaçõesMQTT. O journal registra o que
o processo assinante recebeu e em que fase pode confirmar cada mensagem.
Isso permite retomar uma recepçãoQoS2 depois de reiniciar o client sem
depender somente do estado em memória da biblioteca.

O opt-in é `--subscription-state-dir`. Sem ele, a assinatura mantém seu
comportamento padrão e não ganha persistência local automaticamente.
Publicação tem outboxes próprias: este diretório não cria pipeline de
publicação nem garante transação de negócio.

## Exemplo isolado de laboratório

Com um broker de laboratório já escutando **somente em loopback**:

```sh
mqtt-client sub --open-lab --host 127.0.0.1 --port 1883 \
  --protocol-version 5 --client-id aluno-receptor \
  --clean-start false --session-expiry-seconds 3600 \
  --topic curso/medidas --qos 2 --output jsonl \
  --subscription-state-dir ./dados-assinatura
```

Repita a mesma identidade, destino, protocolo, filtros e opções para retomar.
O broker deve devolver SessionPresent1 para um manifesto já ativo. Se a
sessão sumiu/expirou ou a declaração divergir, o client preserva os arquivos
e recusa a retomada: não reassina silenciosamente uma sessão diferente.

O modo exige Linux/WSL, MQTT5, CleanStartfalse, expiração positiva e IDfixo;
não aceita `--count` nem Will. Saída raw exigecount1 e, portanto, também não
combina com esse modo. Para rede real, use o perfilTLS/mTLS validado em vez
do laboratório aberto; não coloque senhas em comandos/commits.

## O que é gravado e quando ocorre o ACK

1. `Received`: publicação canônica com tópico resolvido, bytes e metadados
   é persistida antes da saída e de qualquer confirmação.
2. Depois de escrever/flush em stdout, ou aplicar os filtros locais, uma
   fase durável autoriza PUBACKQoS1 ou PUBRECQoS2.
3. No QoS2, `AwaitPubRel` aguarda PUBREL; a fase `PubCompPending` deve estar
   durável antes de enfileirar PUBCOMP.
4. Após a emissão do ACK terminal pelo eventloop, o registro pode ser limpo.

Erro de escrita, quota, integridade ou persistência não autoriza confirmar
uma nova obrigação. Identificador de uma obrigação ainda viva não pode
substituir conteúdo diferente. PUBREL retomado pode concluir sem repetir
saída de uma fase já confirmada. Reutilizar ID após conclusão é permitido.

O diretório é privado0700, os arquivos0600, com proprietário verificado,
lock exclusivo, no-follow, checksum, escrita atômica e fsync. O limite é
32registros **e**65536bytes serializados; mensagens grandes podem alcançar
o limite de bytes antes de32. Não compartilhe o diretório entre processos.

O documento inbound atual é versão2. Um draftv1 válido pode ser lido sem
reescrita; a próxima transição durável grava2. A migração verifica checksum
e identidade antigos antes de normalizar a comparação em memória. Os bytes
originais e metadados guardados continuam protegidos pelo checksum completo.
SubscriptionIdentifiers podem variar legitimamente num reenvio e, assim
como expiração restante/alias transitório/DUP, não definem sozinhos uma
nova obrigação. Isso não permite substituir tópico, payload ou UserProperties
de uma obrigação ainda viva. Outboxes de publicação e manifesto mantêm seus
formatos próprios, não foram todos promovidos a2.

## Limites importantes

- Há uma janela entre stdout/flush e o próximo commit. Uma queda nessa
  janela pode repetir saída. ACKMQTT não comprova gravação no sistema
  consumidor nem execução de uma ação física.
- Arquivos do journal contêm payloads e metadados reais. Não os envie para
  GitHub, logs públicos ou materiais do curso; checksum não é criptografia.
- Não apague só o manifesto para tentar resolver uma falha: registros de
  recepção sem o manifesto não devem ser ligados a uma sessão nova. Preserve
  o conjunto e faça diagnóstico/backup offline.
- Prazos são limitados; duplicatas/PING não renovam indefinidamente a espera
  de uma obrigação. Cancelar o processo não apaga o estado da assinatura.
- O checkWindows aprovado verifica compilação, não suporteWindows a esse
  journal. SIGKILL de teste não equivale a perda de energia.

As evidências finais incluem21cenáriosTCP,181testesRust e6regressões de
outbox/WSS na fonte identificada no registro, além de migração com ELFv1 real;
não somar essas contagens como cobertura percentual nem declarar que todo
MQTT5 foi certificado. A [matriz de paridade](mosquitto-parity.md) continua
separando entregas parciais e aceite global.
