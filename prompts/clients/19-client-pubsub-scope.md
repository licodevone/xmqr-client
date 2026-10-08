> **Aviso local C26:** documento histórico da origem. Caminhos/links antigos e agentes citados
> podem não existir neste cliente independente. Use [índice atual](../README.md) e
> [cobertura atual](../COBERTURA.md); não aplique o codec próprio C20 sobre rumqttc.
> Este histórico não é receita de bootstrap nem evidência de execução.

> **Adaptação XMQR — 2026-10-07.** Histórico de escopo da origem, sem prova de execução no XMQR. Não reaplicar sobre o código existente. Para validar o snapshot atual, usar broker/36 e clients/24.
> Leia primeiro [o contrato comum](../contrato-base.md) e a matriz [de cobertura](../COBERTURA.md).
> Origem literal preservada: [arquivo original](../origem/clients/19-client-pubsub-scope.md). Nenhum agente, skill ou MCP externo é requisito instalado.
> O contrato comum prevalece sobre instruções históricas de versão, ambiente e execução. Leitura não autoriza implementação nem execução automática.
# Prompt — escopo do cliente MQTT de laboratório

Este prompt preserva a primeira fatia histórica do cliente CLI Rust MQTT 3.1.1 com modos `pub` e `sub`, QoS 0 e mTLS. Não o use para concluir que o broker ou o cliente atuais continuam limitados a QoS 0. Para reproduzir a versão 0.2.0, use também `22-qos-retained-sessions.md` e `23-interoperabilidade-v020.md`.

## Parâmetros

- Nome do binário: `<mqtt-client; confirmar>`
- Local no workspace: `<src/bin/, crate dedicada ou outro; justificar>`
- Endereço de teste: `127.0.0.1:8883` (somente laboratório)
- Nome TLS esperado/SAN: `127.0.0.1` (não desativar verificação)
- CA confiável: `$HOME/mqtt-lab-certs/ca.crt`
- Certificado/chave do cliente: `$HOME/mqtt-lab-certs/client.crt`, `$HOME/mqtt-lab-certs/client.key`
- Limites: `<tamanho máximo de pacote, tópico, payload, tempo de conexão/leitura/escrita, mensagens em fila>`
- Semântica do `sub`: `<até Ctrl+C, --count N, ou ambos>`
- Nome de cliente e política de reconexão: `<definir>`

## Trabalho pedido

1. Leia as instruções locais, se existirem, `CHANGELOG.md`, `prompts/broker/00-project-context.md`, ADRs e código atual. Confirme por inspeção o que o servidor realmente implementa. O histórico deste prompt começou quando existia apenas transporte mTLS; o estado atual deve vir do código e do `CHANGELOG`, nunca desta nota histórica.
2. Use as orientações de `prompts/contrato-base.md`, `prompts/contrato-base.md`, `prompts/contrato-base.md` e das skills correspondentes. Nesta fatia histórica, limite a implementação a QoS 0; recursos atuais mais avançados pertencem aos prompts 22 e 23 desta pasta.
3. Mapeie os pacotes mínimos: `CONNECT`/`CONNACK`, `SUBSCRIBE`/`SUBACK` e `PUBLISH` QoS 0. Inclua `DISCONNECT` e, se a conexão ficar viva além do Keep Alive, `PINGREQ`/`PINGRESP`. Valide flags fixas, Remaining Length, strings/tópicos, identificador não zero de `SUBSCRIBE`, códigos de retorno e encerramento limpo.
4. Use a especificação [OASIS MQTT 3.1.1](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/mqtt-v3.1.1.html) como fonte normativa. Relacione cada comportamento e teste às seções pertinentes: §2.2–2.3, §3.1–3.3, §3.8–3.9, §3.12–3.14, §4.1 e §4.6–4.7. Verifique os identificadores normativos exatos antes de citá-los na matriz.
5. Declare contratos entre transporte TLS, codec e CLI; diferencie falha TLS, rejeição `CONNACK`, erro de protocolo, timeout, EOF e erro operacional. Não imprima chave privada, payload por padrão ou detalhes sensíveis em erros.
6. Entregue plano de arquivos, exemplos de invocação, matriz de pacotes, critérios de aceite, riscos e itens fora de escopo. Só implemente se o pedido atual autorizar código.

Critério histórico: o cliente QoS 0 troca mensagens com um Mosquitto de referência e com o broker após a fatia `broker/22`. O critério atual completo está em `23-interoperabilidade-v020.md`.
