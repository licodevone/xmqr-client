> **Aviso local C26:** documento histórico da origem. Caminhos/links antigos e agentes citados
> podem não existir neste cliente independente. Use [índice atual](../README.md) e
> [cobertura atual](../COBERTURA.md); não aplique o codec próprio C20 sobre rumqttc.
> Este histórico não é receita de bootstrap nem evidência de execução.

> **Adaptação XMQR — 2026-10-07.** Histórico de escopo da origem, sem prova de execução no XMQR. Não reaplicar sobre o código existente. Para validar o snapshot atual, usar broker/36 e clients/24.
> Leia primeiro [o contrato comum](../contrato-base.md) e a matriz [de cobertura](../COBERTURA.md).
> Origem literal preservada: [arquivo original](../origem/clients/21-client-pubsub-tests.md). Nenhum agente, skill ou MCP externo é requisito instalado.
> O contrato comum prevalece sobre instruções históricas de versão, ambiente e execução. Leitura não autoriza implementação nem execução automática.
# Prompt — testes do cliente pub/sub e integração com o broker

Planeje e execute testes determinísticos para o cliente de `prompts/clients/20-client-pubsub-implementation.md`. Use o agente de protocolo e o de testes/desempenho, com as skills de engenharia de protocolo e conformidade. Registre versão MQTT, precondições, estímulo, resultado observável e cláusula OASIS exata para cada teste.

## Matriz mínima

| Cenário | Evidência esperada |
|---|---|
| mTLS válido, SAN `127.0.0.1` | Handshake estabelecido sem suprimir verificação de nome. |
| CA desconhecida, cliente sem certificado, chave incompatível, SAN incorreto | Falha fechada antes de enviar MQTT; sem segredo em log. |
| `CONNECT` MQTT 3.1.1 / `CONNACK` aceito ou recusado | Só prossegue quando `CONNACK` válido aceita; erro distinguível para recusa. |
| `pub` QoS 0 | Bytes `PUBLISH` corretos; sem Packet Identifier; não promete confirmação de entrega. |
| `sub` QoS 0 | `SUBSCRIBE` com identificador válido; `SUBACK` validado; uma mensagem recebida exibida uma vez. |
| Fragmentação e concatenação de frames | Decoder preserva fronteiras sem leitura excessiva nem perda. |
| Remaining Length hostil, pacote grande, filtro/tópico inválido, flags/códigos indevidos | Erro limitado, sem alocação descontrolada ou panic. |
| Timeout, EOF, fechamento por `Ctrl+C`, Keep Alive | Saída previsível, recursos liberados, sem loop de reconexão acidental. |
| Assinante lento e sequência de mensagens | Limites e backpressure observáveis; ordem do publicador preservada onde aplicável. |

## Ordem e honestidade da evidência

1. Rode unitários com vetores de bytes independentes do encoder próprio e servidor TLS/MQTT simulado para estados e falhas.
2. Teste o cliente contra Mosquitto de versão fixada, com mTLS exigido, executando `sub` e depois `pub` em processos distintos. Registre versão, configuração relevante e comandos exatos; proteja chaves privadas. Um sucesso somente contra o próprio broker não prova interoperabilidade.
3. Teste contra o broker local **após** confirmar que ele implementa `CONNECT`/`CONNACK`, `SUBSCRIBE`/`SUBACK`, `PUBLISH` QoS 0 e roteamento. Se continuar sendo apenas listener TLS, marque o teste pub/sub como **não implementado no servidor**, não como falha do cliente nem como aprovado.
4. Faça o teste cruzado `mosquitto_pub/sub` → broker e cliente próprio → Mosquitto. Compare comportamento observável com a [especificação MQTT 3.1.1 OASIS](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/mqtt-v3.1.1.html); divergência do Mosquitto não é automaticamente bug normativo.
5. Registre resultados na matriz de conformidade como `aprovado`, `falhou`, `não implementado` ou `bloqueado`, com referência de seção/cláusula. Inclua comandos de reprodução e testes de regressão para toda correção.

Critério de aceite desta fatia: os modos `pub` e `sub` interoperam com broker externo mTLS no escopo QoS 0 e resistem aos casos negativos acima. O aceite de ponta a ponta com **este** broker é um marco separado, condicionado à implementação do protocolo no servidor.
