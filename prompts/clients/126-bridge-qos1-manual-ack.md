# C126 — bridge QoS 1 com ACK da origem após destino

Entrada: C125 publicado em `main` sem release, [contrato P224](../../../xmqr/prompts/broker/224-bridge-qos1-contract.md). Este prompt precede código funcional.

Objetivo: adicionar `route.qos=1` opcional, mantendo `qos=0` como padrão compatível. No modo 1, usar assinatura com sessão MQTT 3.1.1 persistente e ACK manual da origem; encaminhar QoS 1 ao destino e confirmar a origem somente após PUBACK correspondente do destino. Limitar a uma publicação em voo e timeout explícito. Preservar retained na saída QoS 1.

Segurança/falha: sem destino conectado, aguardar reconexão dentro do limite, sem ACK da origem; após timeout ou erro, encerrar sem confirmação para permitir redelivery. Após crash/restart do sidecar, a mesma identidade deve receber mensagem pendente e encaminhá-la. Duplicação depois de PUBACK do destino e antes do ACK da origem é admissível; não alegar exatamente uma entrega. Não registrar payload, segredos ou caminho privado.

Aceite: testes reais XMQR↔Mosquitto e XMQR↔XMQR com QoS 1, retained, queda/reconexão e crash entre confirmações, negativos de credencial/ACL, regressão QoS 0 C125, fmt/test/Clippy/licença/CI. Entrega parcial da fase 8, sem tag antes da distribuição validada.
