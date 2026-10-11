# C38 rev1 — limites e diagnósticos da fase 1

## rev5 — análise Rust1.99 no Ubuntu26

Clippy recusou assert!(output.stdout.is_empty()) em tests/wire.rs:882.
Usar comparação do comprimento comzero: mantém teste e evita imprimir payload
na falha. Sem mudar implementação ou afrouxar ausência de saída. Repetir gates.

Entrada: HEAD 6ee5aa87d0e1eb16c48c625408eeaf227197da14, versão0.6.0/tag local v0.6.0, árvore limpa. AGENTS.md, README, ORIGIN.json, skill xmqr-client-development e plano de evolução lidos. Pedido autoriza execução; commit/tag/publicação coordenados pelo agente raiz após gates.

Objetivo: fixar evidência nas fronteiras de payload4096 e pacote completo65536 bytes; esclarecer recusas SUBACK/transporte sem inferir quotas de uma desconexão MQTT3.1.1. C37 mantém timeout/keepalive.

Escopo: src/cli.rs e src/session.rs para mensagens estáticas claras; tests/wire.rs e unitários para N/N+1 em UTF-8/bytes e SUBSCRIBE completo; script de integração Unix isolado com broker independente explícito; README, CHANGELOG, matriz e registroC38. Sem fase2, bump independente ou persistência local.

Contrato: limite pacote inclui primeiro byte, codificação Remaining Length e corpo. Tópico/filtro1024 bytes, até256 filtros, payload4096 permanecem. SUBACK0x80 não distingue ACL/quota; PUBACK/PUBCOMP não informam reason de quota em MQTT3.1.1. Não exibir payload, segredos, caminhos privados nem inventar causas. TLS/SAN, lab loopback e retry/reenvio preservados. Sem migração de estado.

Aceite: payload4096 enviado e4097 recusado antes de rede; SUBSCRIBE65536 aceito e65537 recusado antes de rede; negativos UTF-8; SUBACK recusado explica ACL/limites possíveis e ausência de reason específico; perda de transporte conserva resultado desconhecido sem afirmar quota. Gates fmt/test/clippy/build Windows; Unix e integração real separados BLOCKED quando sem distribuição WSL. Integração isolada deve cobrir pub/sub4096 QoS0/1/2, keepalive1 mantendo assinatura e timeout selecionado, broker real obrigatório. Nenhuma fixture equivale ao aceite Unix. Revisar prompt após falhas antes de corrigir.

## Rev2 — parser rumqttc

Inspeção local da dependência detectou que set_max_packet_size incoming compara Remaining Length; outgoing compara pacote completo. Ajustar incoming para65532 (=65536 menos header4 na fronteira). Valores inferiores65532 têm cabeçalho no máximo4 e permanecem aceitos; Remaining Length65533 implica pacote65537 e deve ser recusado por header antes de corpo. Adicionar fixture wire recebida65536/65537. Mensagem local de pacote recebido excedido deve citar64KiB sem bytes privados.

## Rev3 — Clippy e recusa por transporte

Clippy inicial FAIL: três casts de fixture podem truncar/perder sinal; substituir por try_from com unwrap sobre valores matematicamente limitados antes de repetir gates. Inspeção do broker confirma recusa de quotas de assinaturas encerra transporte, sem SUBACK de reason; harness deverá conferir erro genérico sem sucesso/reenviar e sem atribuir quota a EOF. SUBACK0x80 continua coberto por fixture estática de política. Encoding não mínimo de Remaining Length do rumqttc requer decisão de escopo e limitação explícita, não anunciar teto completo absoluto com parser atual.

## Rev4 — cap conservador aprovado pelo coordenador

Decisão: sem vendor/fork nesta fatia. Configurar entrada rumqttc com Remaining Length máximo65531, garantindo pacote recebido até65536 mesmo com header de5bytes/não mínimo. SaídaSUBSCRIBE permanece65536 inclusivo. Entrada canônica65536 (corpo65532/header4) pode ser recusada1byte antes; declarar exceção no README/matriz/registro, sem prometer fronteira inclusiva idêntica. Tráfego suportado pelo broker, payload4096/tópico1024, fica abaixo desse cap. Testar header mínimo/não mínimo65531/65532, erro antes do corpo, stdout vazio/sem payload privado. Diagnóstico deve citar limite local de recepção, sem afirmar que todo packet recusado excede65536. Aceite Unix ainda depende execução real e não é liberado por essa decisão.

## Rev5 — drenagem de saída no harness

Revisão do coordenador identificou risco de deadlock: JSON de payload4096 produz cerca de15KiB, maior que alguns pipesUnix; wait antes de stdout.read pode bloquear o client. Antes da correção, revisar harness para stdout em arquivo temporário pertencente ao laboratório, eliminando backpressure. Stderr permanece com um único leitor de readiness. A leitura do arquivo ocorre após wait bounded; cleanup fecha handles antes de remover diretório. Não alterar Rust; repetir AST/guardWindows, sem promover integração real a PASS.
