# C102 — User Property em SUBSCRIBE/UNSUBSCRIBE MQTT 5

Entrada: o broker P180 aceita pares repetidos de User Property em SUBSCRIBE e UNSUBSCRIBE com limite 16/2048 bytes. O client 0.27.0/C101 só envia Subscription Identifier e opções de assinatura; não expõe os pares de propriedade nesses dois pacotes. Este prompt precede o código.

Objetivo: acrescentar `--subscribe-user-property CHAVE=VALOR` e `--unsubscribe-user-property CHAVE=VALOR`, repetíveis, somente em `sub --protocol-version 5`. Validar UTF-8 MQTT, campos até 256 bytes, no máximo 16 pares/2048 bytes por contexto antes da conexão. Preservar ordem e combinação com Subscription Identifier; enviar as propriedades em cada SUBSCRIBE e em cada UNSUBSCRIBE efetuado por `--count`. Sem ecoar conteúdo remoto na saída. Nenhum segredo deve ser passado nessas flags.

Aceite: unitários de CLI/wire para positivos, repetição e negativos; integração real com broker P180 para entrega, confirmação de remoção e regressão de QoS/3.1.1; fmt, Rust, Clippy, build release, licenças, distribuição e CI. Atualizar uso, docs, checklist cruzado e registro parcial 6A/6D. Sem nova tag isolada nem migração.

Revisão 2: o primeiro gate parou no `cargo fmt --check` das novas linhas, antes dos testes. Formatar a asserção e a atribuição e repetir a verificação completa antes de aceitar a funcionalidade.

Revisão 3: o teste unitário passou; a fixture TCP comparou o DISCONNECT normal a um corpo de um byte, mas a codificação abreviada válida enviada pelo rumqttc tem Remaining Length zero. Corrigir só a expectativa do teste, preservar o wire do client e repetir os gates.

Revisão 4: a integração broker/client chegou a um timeout aguardando a linha de SUBACK na observação por `stderr`. O leitor Python pode manter uma segunda linha no buffer após `readline`, enquanto `select` observa apenas o descritor. Tornar o pipe não-bufferizado para distinguir falha real do client de falha do observador e repetir; se o prazo persistir, coletar o diagnóstico do processo antes de alterar o protocolo.
