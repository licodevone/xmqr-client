# Paridade do mqtt-client com Mosquitto

## Fase3 — C45 / C07, 2026-10-13

[Robustez](client-robustness.md): totalmonotônico opcional1..86400,
protocolo/idle/input/cancelamento separados, jitter80..100% da base exponencial,
retrybudget0..10semreset e cancelamento prioritário. QoS1duplicata legítima e
QoS2recebido incerto mantêm suas limitações; nenhuma persistência local nova.
O [registroC45](../prompts/registros/C45-robustez-reconexao.md) separa fixtures/gates
de interop externaP91, publicação e aceitedafase3 global.

## Fase2 — C43, 2026-10-13

[Checklist próprio](phase2-acceptance.md):59cenários reais mTLS com
Mosquitto2.0.22 pinado PASS. Clientruntime0.8.1 preservado. Fase compartilhada
validada no software, BLOCKED paraE01 físico. Tabelas históricasC36 abaixo
não substituem resultados novos de C43.

## Planejamento concluído e distribuição — C42, 2026-10-13

Client 0.8.1 é um patch de distribuição: contexto Docker com allowlist,
conferência da versão real/labels e artefatos/proveniência para Docker Hub.
Sem mudança nas opções, filtros retained, wire, limites ou credenciais da 0.8.0.
O [plano detalhado compartilhado](https://github.com/licodevone/xmqr/blob/main/docs/plano-paridade-mosquitto.md)
define C01–C08, MQTT5, transports e distribuição com dependências e aceite;
as fases futuras não foram implementadas nesta entrega.

Atualizado em C41, 2026-10-12: client 0.8.0, independente MQTT3.1.1/rumqttc.
Filtros retained aceitos em fixtures e integração própria QoS0/1/2. As tabelas
de validação C36 abaixo são históricas; aceite posterior da fase1 é P61/C39.
Campanha de capacidade broker/client: [P72](../../xmqr/docs/performance-fanout.md).
Esta matriz não declara conformidade integral ou paridade exaustiva.

## Matriz funcional

| Capacidade | XMQR atual | Lacuna/próximo incremento | Evidência |
| --- | --- | --- | --- |
| pub/sub QoS0/1/2 | --qos | Interop externa ampliada | src/session.rs; tests/wire.rs |
| Payload texto/binário/stdin | --message, --message-file, --stdin, --line-mode; --repeat/--repeat-interval-ms para fonte única | Sem replay de resultado desconhecido | src/payload.rs; src/cli.rs; C53 |
| Assinaturas múltiplas | --topic repetido, --count; filtros retained e --exclude-topic na saída | Filtragem local não altera ACL ou assinatura | src/cli.rs; tests/wire.rs; C52 |
| Retained e Will | --retain, --will-message e --will-message-file binário | Limite 4096 bytes; MQTT5 fora do escopo | src/cli.rs; src/session.rs; C55 |
| Sessão/reconexão | --clean-session, --client-id, --reconnect-attempts | Persistência local/reconexão avançada exige contrato | src/session.rs; tests/wire.rs |
| TLS/mTLS | CA/cert/chave e senha; SAN validado | TLS sem certificado cliente | src/tls.rs; src/credentials.rs |
| Diagnóstico e saída | text/jsonl/hex/raw; raw exige --count 1 | Sem timestamp de publicação fabricado | src/output.rs; C54 |
| Keepalive | --keepalive-seconds 1..65535, default30 | Zero fora do contrato | src/session.rs; tests/wire.rs |
| Timeouts | --timeout-seconds 1..300, default8; idle90s preservado | Não é timeout total/espera de aplicação | src/session.rs; registroC37 |
| Perfis reutilizáveis | CLI/algumas variáveis, configuração systemd | Prompt próprio de perfis | README; packaging |
| MQTT5/WS/WSS | MQTT5 pub/sub QoS0/1/2, opções de sessão, propriedades PUBLISH C77/C78, JSONL C80, repetição C81, Topic Alias C83, Retain Handling C84, User Property de SUBSCRIBE/UNSUBSCRIBE C102, Session Expiry no DISCONNECT C104, outbox C105/C107 para um QoS 1 ou 2 de saída, manifesto C108 para retomar assinatura e ACK após flush C110 no Linux/WSL; C112 adiciona WS em loopback, C114 WSS mTLS local para pub/sub MQTT 3.1.1/5, C116 perfil TOML WSS mTLS local, C117 opt-in WSS remoto, C119/C120 outbox QoS 1 por WSS e retomada, C122 outbox QoS 2 por WSS, C123 manifesto de assinatura por WSS em `main` | Navegador real, mensagens recebidas sem journal local, múltiplos inflight, reinício do broker e matriz completa de propriedades/opções pendentes | src/cli.rs; src/profile.rs; src/session.rs; src/v5_session.rs; C57–C123 |

Mosquitto oferece opções de protocolo, keepalive, repetição e propriedadesMQTT5; sub oferece timeout, exclusão de tópicos e filtros retained. Referências consultadas: [pub](https://mosquitto.org/man/mosquitto_pub-1.html), [sub](https://mosquitto.org/man/mosquitto_sub-1.html). Há opções específicas de2.1; registrar mosquitto --version/compilação antes da campanha. Não assumir equivalência a toda versão instalada.

## Validação cruzada planejada

| Direção/cenário | Evidência exigida | Estado C36 |
| --- | --- | --- |
| Client XMQR → Mosquitto | QoS0/1/2, retained/Will, clean/persistent, reconnect e erros | NOT_RUN nesta tarefa |
| mosquitto_pub/sub → XMQR | Mesmos cenários; TLS/mTLS e ACL negativos | NOT_RUN nesta tarefa |
| Client XMQR → XMQR P55 | QuotasN/N+1, recusa/redução/restart sem perda | BLOCKED: aceite Unix P55 pendente |
| Fixtures wire do client | CONNECT/SUBACK/ACK/erros/cancelamento sem estado real | C37: fixtures locais; ver registro de gates |

Preservar payload4096, tópico1024 e laboratórios loopback. SUBSCRIBE de saída conta pacote completo65536 inclusivo; recepção rumqttc usa cap conservador de corpo65531 para garantir frame até65536 mesmo com header5. Pode recusar frame canônico65536 por1byte; não afeta tráfego broker suportado4096/1024 e não é paridade inclusiva da fronteira de entrada. ACK não comprova processamento de negócio. Sessão persistente no broker não comprova persistência local do client.

## Sequência de evolução

1. C37 implementado: timeout de protocolo e keepalive configuráveis; ver registro de gates e limites.
2. Quotas/payload/pacote: integração contra P55 aceito e avaliação da fase1; não aumentar client unilateralmente.
3. Perfis de conexão e filtros retained/saída em prompts independentes.
4. Modalidades TLS, repetição controlada e interoperabilidade externa conforme demanda.
5. MQTT5 e WS/WSS coordenados com o [plano do broker](../../xmqr/docs/evolution-plan.md).

Criar prompt antes de funcionalidade, registrar evidência, validar commit exato e criar tag de marco aceita. Tags de versão devem acompanhar manifests e conteúdo efetivo; releases incluem pacotes testados/checksums. C36 é marco documental, sem novo pacote.

C38: negativos locais de payload/packet e diagnóstico de SUBACK sem inferir causa; harness Unix real `scripts/verify_phase1.py` preparado. [RegistroC38](../prompts/registros/C38-limites-fase1.md) separa gates portáveis e integração BLOCKED; fase1 não aceita por fixtures.

## Checkpoint atual — aceite P61/C39, 2026-10-11

Fase1 concluída no Ubuntu26/WSL: broker0.11.0/client0.7.0, configuração e
quotas, limites de pacotes, integrações e pacote install/upgrade/rollback aceitos.
Os BLOCKED anteriores desta página são históricos. Ver [registroC39](../prompts/registros/C39-release-fase1.md).
Fase2 (capacidade/RAM/latência) aguarda prompt próprio e não foi executada aqui.
