# C102 — User Property em SUBSCRIBE/UNSUBSCRIBE MQTT 5

Entrada: o broker P180 aceita pares repetidos de User Property em SUBSCRIBE e UNSUBSCRIBE com limite 16/2048 bytes. O client 0.27.0/C101 só envia Subscription Identifier e opções de assinatura; não expõe os pares de propriedade nesses dois pacotes. Este prompt precede o código.

Objetivo: acrescentar `--subscribe-user-property CHAVE=VALOR` e `--unsubscribe-user-property CHAVE=VALOR`, repetíveis, somente em `sub --protocol-version 5`. Validar UTF-8 MQTT, campos até 256 bytes, no máximo 16 pares/2048 bytes por contexto antes da conexão. Preservar ordem e combinação com Subscription Identifier; enviar as propriedades em cada SUBSCRIBE e em cada UNSUBSCRIBE efetuado por `--count`. Sem ecoar conteúdo remoto na saída. Nenhum segredo deve ser passado nessas flags.

Aceite: unitários de CLI/wire para positivos, repetição e negativos; integração real com broker P180 para entrega, confirmação de remoção e regressão de QoS/3.1.1; fmt, Rust, Clippy, build release, licenças, distribuição e CI. Atualizar uso, docs, checklist cruzado e registro parcial 6A/6D. Sem nova tag isolada nem migração.
