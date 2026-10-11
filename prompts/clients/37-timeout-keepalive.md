# C37 rev4 — timeout e keepalive configuráveis

Entrada: HEAD `5d1f74207f953d31b9a246c8626bfacf359f1dd9`, versão 0.5.0; alteração preexistente em `.serena/project.yml` preservada. Broker P55 com aceite Unix pendente não é requisito para fixtures locais do client. Pedido atual autoriza preparar prompt, implementar e validar; autorização explícita posterior permite versão 0.6.0, commit/push e tag após aceite. Coordenação do commit/tag/push será realizada pelo agente raiz. Aplicar xmqr-client-development.

Objetivo: substituir fixos keepalive30s e timeout de protocolo8s por --keepalive-seconds1..65535/default30 e --timeout-seconds1..300/default8. Sem zero/ilimitado. Timeout atua na espera por CONNACK (por poll/tentativa, preservando orçamento de retry), SUBACK, conclusão PUBLISH/QoS2 e DISCONNECT normal. Não é tempo total do comando nem espera por mensagem de aplicação. Inatividade de assinatura90s/input8s/cancelamento1s preservados.

Normativo: OASISMQTT3.1.1 §3.1.2.10, keepalive em segundos16bits, PINGREQ e janela do broker1,5x; rumqttc0.25.1 suporta valores em segundos. Zero é válido no protocolo mas fora deste contrato positivo. Não alterar QoS/Will/ACL/TLS, reconexão/reenvio, credenciais ou quotas. Alterações: src/cli.rs, src/session.rs, tests/wire.rs, README/changelog/matriz, manifest/lock e registroC37.

Aceite: defaults preservados; limites inferiores/superiores e inválidos/repetidos/faltantes; CONNECT carrega keepalive selecionado em bytes; broker silencioso/sem SUBACK/PUBACK expira conforme timeout; reconexão mantém opções; fmt/test/clippy/build e fixtures isoladas. Registrar Unix/external NOT_RUN se indisponível. Atualizar manifest/lock para 0.6.0 após aceite dos gates portáveis. Tag imutável v0.6.0 e commit/push apenas pelo coordenador após revisão. Não afirmar pacote Ubuntu novo ou aceite Unix. Registrar Windows e condições reais dos gates.

Fonte: https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html. Registrar falhas e revisar prompt antes de corrigir. Não executar próxima funcionalidade só por existência do planejamento.

Fontes oficiais consultadas em 2026-10-11: [OASIS MQTT 3.1.1 §3.1.2.10](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html) e [rumqttc 0.25.1 MqttOptions](https://docs.rs/rumqttc/0.25.1/rumqttc/struct.MqttOptions.html#method.set_keep_alive).

## Rev4 — pacote do marco aceito

Autorização explícita atual: commit/push e tag a cada evolução aceita, release quando necessária. Antes da tag, validar também cargo test --locked --release e cargo build --locked --release --bins, empacotar o executável Windows portátil com LICENSE/notas, gerar SHA-256 e verificar --version/--help após extração. Anexos em target/c37-release (ignorado), sem segredos/estado. A tag 0.6.0 representa C37 com gates portáveis e pacote Windows; não homologa Windows como serviço nem anuncia pacote Ubuntu, integração externa, aceite P55 ou encerramento fase1. Publicar release distribuída somente quando os pacotes das plataformas pretendidas estiverem validados; nesta retomada o pacote Ubuntu está bloqueado. Registrar autenticação de push indisponível sem afirmar publicação.
