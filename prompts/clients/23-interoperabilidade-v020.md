> **Aviso local C26:** documento histórico da origem. Caminhos/links antigos e agentes citados
> podem não existir neste cliente independente. Use [índice atual](../README.md) e
> [cobertura atual](../COBERTURA.md); não aplique o codec próprio C20 sobre rumqttc.
> Este histórico não é receita de bootstrap nem evidência de execução.

> **Adaptação XMQR — 2026-10-07.** Histórico de escopo da origem, sem prova de execução no XMQR. Não reaplicar sobre o código existente. Para validar o snapshot atual, usar broker/36 e clients/24.
> Leia primeiro [o contrato comum](../contrato-base.md) e a matriz [de cobertura](../COBERTURA.md).
> Origem literal preservada: [arquivo original](../origem/clients/23-interoperabilidade-v020.md). Nenhum agente, skill ou MCP externo é requisito instalado.
> O contrato comum prevalece sobre instruções históricas de versão, ambiente e execução. Leitura não autoriza implementação nem execução automática.
# Prompt — gate de interoperabilidade do cliente e broker 0.2.0

Valide a combinação cliente Rust + broker 0.2.0 sem usar apenas o codec próprio como oráculo.

## Matriz mínima

- Cliente Rust ↔ broker deste projeto.
- `mosquitto_pub/sub` ↔ broker deste projeto.
- Cliente Rust ↔ Mosquitto de versão fixada.
- ESP-MQTT ou outra biblioteca independente ↔ broker deste projeto, quando o ambiente estiver disponível.

## Cenários

CONNECT/autenticação, ACL, QoS 0/1/2, retransmissão, retained, sessão `CleanSession=0`, fila offline, restart, takeover, credencial inválida, tópico negado e limites.

Registre versões, comandos parametrizados, precondições, estímulo, resultado observável e cláusula OASIS. Use certificados diferentes por identidade e não publique chaves/segredos no relatório.

## Gate

Um cenário sem cliente externo disponível é `BLOCKED`. Divergência deve gerar diagnóstico e caso de regressão; não ajuste o teste para aceitar comportamento incompatível sem justificativa normativa.
