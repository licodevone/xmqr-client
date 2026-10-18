# C139 v1.0 — origem MQTT 5/QoS 2 na bridge com journal isolado

## Entrada

`main` em `a720399`, release client 0.41.0/bridge 0.3.0 publicada da
fonte `1d60d38`; CI final `39170753201` PASS. C136/C137 oferecem origem
MQTT 5 em QoS 0/1 com Retain As Published. C134 oferece QoS 2 com origem
MQTT 3.1.1 e journal privado. A saída local não rastreada
`vendor/rumqttc-0.25.1/target/` deve ser preservada.

## Objetivo

Aceitar `source_protocol="mqtt5"` e `route.qos=2` mantendo destino MQTT
3.1.1. A assinatura MQTT 5 usa QoS 2 e Retain As Published; retained
anterior/vivo e publicação comum devem chegar com o bit correto. Após
PUBCOMP do destino, registrar obrigação durável na origem antes do PUBREC;
ao receber PUBREL, confirmar conclusão durável antes de PUBCOMP. Em crash,
retomar somente com sessão de origem presente e mesma identidade de
rota/protocolo. O journal MQTT 5 não pode ser interpretado como journal
MQTT 3.1.1, nem vice-versa.

Referência normativa: [MQTT 5.0 OASIS](https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html),
seções de QoS 2, estado de sessão e Retain As Published. Isto não é
transação entre brokers nem entrega de negócio exatamente uma vez.

## Transições e falhas

- PUBLISH QoS 2 da origem: validar tópico, tamanho, ID e rota; se já
  registrado com mesma identidade/conteúdo, não reenviar ao destino.
  Se novo, encaminhar uma vez ao destino QoS 2 e aguardar PUBCOMP com
  prazo; gravar journal e somente então enviar PUBREC à origem.
- PUBREL da origem: exigir obrigação no journal, concluir/persistir e
  então enviar PUBCOMP por API manual. Repetição após crash deve ser
  tratada de modo idempotente ou recusada explicitamente sem afirmar
  entrega; qualquer janela residual deve ficar no registro.
- Broker de origem sem Session Present com obrigação pendente: falhar
  fechado preservando journal. Destino QoS 2 usa sessão persistente;
  queda após PUBCOMP e antes do registro pode duplicar entre brokers.
- Configuração sem `journal_dir` privado, identidade trocada, arquivo
  inseguro/corrompido, ACK incorreto, ACL/TLS/CA/SAN/senha inválidos
  falham sem confirmar origem nem descartar estado. Senha/payload não
  aparecem no log. Default MQTT 3.1.1 e MQTT 5/QoS 0/1 não mudam.

## Escopo e aceite

Arquivos: bridge, journal quando necessário, fixtures XMQR e Mosquitto,
CI, guia e registro C139. Sem alteração de versão/pacote/systemd nesta
fatia. Testar replay retained, retained vivo e publicação comum; dois
checkpoints de crash em debug, retomada sem duplicação indevida no ponto
durável, perda de sessão, destino reiniciado e negativos de configuração.
Regressão MQTT 3.1.1 QoS 0/1/2 e MQTT 5 QoS 0/1; fmt/test/clippy/build,
inventário e CI exata. Registrar FAIL/BLOCKED e limites honestos; T02
permanece aberto para propriedades, ciclos externos e semântica ampliada.
