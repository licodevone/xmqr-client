# C123 — manifesto de assinatura persistente MQTT 5 por WSS mTLS

Entrada: client 0.35.0 publicado; C108/C110 persistem uma assinatura MQTT 5 por TCP e confirmam mensagens QoS 1/2 após escrita/flush. C119/C122 usam URL WSS canônica como identidade de outbox. Assinatura com `--subscription-state-dir` por `--ws-url` ainda é recusada.

Objetivo: permitir uma assinatura contínua MQTT 5 por WSS mTLS local/remoto com manifesto privado, Client ID fixo, Clean Start false e Session Expiry positivo. Gravar URL WSS canônica no campo `host` existente e porta sentinela 0, sem migração; manifestos TCP permanecem intactos. Após SUBACK de êxito, persistir `Active`; se sessão confirmada, retomar sem novo SUBSCRIBE; sem Session Present, preservar manifesto e falhar sem assumir assinatura ativa. QoS 1/2 recebido mantém ACK depois de output/flush. WS aberto, `--count` e múltiplas identidades/destinos continuam recusados.

Aceite: parser local/remoto, isolamento de URL/TCP e negativos; fixture WSS mTLS com primeira assinatura, SUBACK incerto, retomada com/sem Session Present, mensagem/ACK após saída e destino divergente; integração com broker real, regressões TCP/outbox QoS 1/2, suíte/fmt/Clippy/licenças/CI da fonte exata. Registrar limite de saída stdout e possibilidade de duplicação após crash. Sem tag/release isolada.
