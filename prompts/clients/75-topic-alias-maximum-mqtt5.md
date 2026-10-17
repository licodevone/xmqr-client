# C75 rev2 — limite de Topic Alias recebido no client MQTT 5

Entrada: client 0.18.0, broker P139 em validação. Criar prompt antes do código. A biblioteca rumqttc 0.25.1 implementa `MqttOptions::set_topic_alias_max` e tabela de aliases recebidos por conexão; `mosquitto_sub` 2.0.22 foi isolado com erro na forma abreviada, então é necessária validação independente.

Objetivo: permitir `mqtt-client sub --protocol-version 5 --topic-alias-maximum 0..16` no CONNECT. Zero desativa, ausência mantém o padrão MQTT5 zero; não oferecer em MQTT3.1.1 nem em `pub`. Enviar a propriedade `0x22` exatamente uma vez, validar antes da rede, deixar o transporte MQTT5 rumqttc expandir nomes recebidos e manter filtros/saída sobre o tópico resolvido. Reconexão começa sem mapa antigo; não prometer Topic Alias na publicação do client.

Aceite: parser/negativos, wire CONNECT, integração real com broker P139 para estabelecimento, nome abreviado, QoS0/1/2, reconexão e limite; regressão client e Mosquitto no fluxo sem alias, fmt/test/Clippy/build/CI, documentação e checklist honestos. Commit/push após evidência, sem tag/release para fatia isolada.

Revisão após o primeiro gate: 53 testes de CLI PASS; o teste wire novo falhou porque a saída da segunda mensagem trouxe tópico vazio. Na rumqttc 0.25.1, `handle_incoming_packet` enfileira uma cópia do evento antes de expandir o alias internamente. O client XMQR deve manter sua própria tabela limitada à conexão para exibição/filtros, atualizada por mensagens com nome e alias e usada quando o nome recebido estiver vazio. Rejeitar alias fora do máximo, zero, desconhecido e tópico vazio sem alias. Testar reset em reconexão. Não atribuir essa expansão à biblioteca.

Revisão após a integração: o client atual encerra `sub` em erro de transporte, sem retomada automática no mesmo processo. O teste de reset deve abrir um novo processo/conexão e confirmar que o alias anterior não é aceito; não anunciar reconexão automática. Confirmar negativos com fixture wire antes de aceitar C75. O Mosquitto 2.0.22 fecha ao receber a forma abreviada válida até em fixture independente do XMQR; registrar a limitação externa, sem alegar interoperabilidade desse fluxo.

Revisão do gate final: após 53 testes de CLI e 48 wire PASS, Clippy reprovou `assert!(output.stdout.is_empty())` no novo teste negativo (`assert_is_empty`). Trocar pela asserção `assert_eq!` sugerida, que mostra a saída inesperada, e repetir fmt, suíte, Clippy e build.
