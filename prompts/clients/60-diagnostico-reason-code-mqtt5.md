# C60 rev1 — diagnóstico MQTT 5 de recusa de publicação e assinatura

Data: 2026-10-16. Entrada: `main` `5eab7d4`, client 0.14.0 experimental, C57/C58 aceitos. Entrega coordenada com P119, sem nova versão nesta fatia.

## Objetivo e escopo

Exibir no erro do comando `mqtt-client pub --protocol-version 5 --qos 1` o reason code real de PUBACK recusado, incluindo `NotAuthorized` (`0x87`) do broker P119, e preservar a indicação de resultado desconhecido em falha de transporte. Em `sub --protocol-version 5`, indicar reason code de SUBACK por filtro recusado, sem imprimir credenciais ou payload. Arquivos: `src/v5_session.rs`, testes pertinentes, README, changelog e registro C60.

## Limites e aceite

Não confundir PUBACK com sucesso da aplicação consumidora, não reenviar automaticamente, não alterar MQTT 3.1.1, quotas, TLS ou saída de publicação bem-sucedida. Validar erros de recusa reais contra broker P119 e SUBACK recusado; regressão pub/sub positiva. Gates fmt/test/clippy/build e CI. Sem bump/tag isolados.

Fonte normativa: <https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html>, §§3.4.2 e 3.9.3.

Rev2: o teste de wire de PUBACK recusado falhou com `Connection closed by peer abruptly` porque o servidor simulado encerrou o socket imediatamente após escrever a resposta. Manter o socket aberto até o client processar o PUBACK e fechar a conexão, então repetir o teste. Não modificar o tratamento de erro em produção por causa de um encerramento prematuro da fixture.

Rev3: a primeira execução da integração real falhou antes da rede porque o identificador gerado pela fixture excedeu o limite de 23 caracteres do client. Usar IDs curtos e distintos e repetir a integração completa; não alterar a quota do produto.

Rev4: Clippy all-targets falhou em dois avisos novos: `assert_is_empty` no teste e `trivially_copy_pass_by_ref` na função de SUBACK. Usar assert com bytes esperados e passar o enum pequeno por valor; repetir os gates. O teste funcional e a integração real já passaram.

Rev5: revisão do diagnóstico identificou que um SUBACK de sucesso com QoS diferente é alteração de QoS, não recusa. Distinguir os textos e testar o downgrade separadamente.
