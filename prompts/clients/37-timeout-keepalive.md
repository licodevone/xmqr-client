# C37 rev1 — timeout e keepalive configuráveis

Entrada: client1f22a18/C36, versão0.5.0; árvore limpa. BrokerP55 com aceite Unix pendente não é requisito para fixtures locais do client. Pedido autoriza implementar, commit/push e tag de cada evolução aceita. Aplicar xmqr-client-development; Serena ativo no client, instruções iniciais lidas.

Objetivo: substituir fixos keepalive30s e timeout de protocolo8s por --keepalive-seconds1..65535/default30 e --timeout-seconds1..300/default8. Sem zero/ilimitado. Timeout atua na espera por CONNACK (por poll/tentativa, preservando orçamento de retry), SUBACK, conclusão PUBLISH/QoS2 e DISCONNECT normal. Não é tempo total do comando nem espera por mensagem de aplicação. Inatividade de assinatura90s/input/cancelamento1s preservados.

Normativo: OASISMQTT3.1.1 §3.1.2.10, keepalive em segundos16bits, PINGREQ e janela do broker1,5x; rumqttc0.25.1 suporta valores em segundos. Zero é válido no protocolo mas fora deste contrato positivo. Não alterar QoS/Will/ACL/TLS, reconexão/reenvio, credenciais ou quotas. Alterações: src/cli.rs, src/session.rs, tests/wire.rs, README/changelog/matriz, manifest/lock e registroC37.

Aceite: defaults preservados; limites inferiores/superiores e inválidos/repetidos/faltantes; CONNECT carrega keepalive selecionado em bytes; broker silencioso/sem SUBACK/PUBACK expira conforme timeout; reconexão mantém opções; fmt/test/clippy/build e fixtures isoladas. Registrar Unix/external NOT_RUN se indisponível. Versionar0.6.0 após aceitar gates portáveis; tag anotada v0.6.0 no commit exato, sem afirmar pacoteUbuntu novo. Release com pacote somente após build/install/upgrade pertinentes.

Fonte: https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html. Registrar falhas e revisar prompt antes de corrigir. Não executar próxima funcionalidade só por existência do planejamento.
