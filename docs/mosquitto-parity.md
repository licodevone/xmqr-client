# Paridade do mqtt-client com Mosquitto

Checkpoint C38, 2026-10-11. Client independente MQTT3.1.1/rumqttc. Pacote0.5.0 experimental; C35 está no código posterior à release. Esta matriz é documental, não conformidade integral ou benchmark.

## Matriz funcional

| Capacidade | XMQR atual | Lacuna/próximo incremento | Evidência |
| --- | --- | --- | --- |
| pub/sub QoS0/1/2 | --qos | Interop externa ampliada | src/session.rs; tests/wire.rs |
| Payload texto/binário/stdin | --message, --message-file, --stdin, --line-mode | Repetição com intervalo | src/payload.rs; src/cli.rs |
| Assinaturas múltiplas | --topic repetido, --count | Filtros de saída/retained | src/cli.rs; tests/wire.rs |
| Retained e Will | --retain, --will-* | Filtros retained; Will binário conforme demanda | src/cli.rs; src/session.rs |
| Sessão/reconexão | --clean-session, --client-id, --reconnect-attempts | Persistência local/reconexão avançada exige contrato | src/session.rs; tests/wire.rs |
| TLS/mTLS | CA/cert/chave e senha; SAN validado | TLS sem certificado cliente | src/tls.rs; src/credentials.rs |
| Diagnóstico e saída | text/jsonl e erros estáticos | Filtros/formatos conforme demanda | src/output.rs; registroC35 |
| Keepalive | --keepalive-seconds 1..65535, default30 | Zero fora do contrato | src/session.rs; tests/wire.rs |
| Timeouts | --timeout-seconds 1..300, default8; idle90s preservado | Não é timeout total/espera de aplicação | src/session.rs; registroC37 |
| Perfis reutilizáveis | CLI/algumas variáveis, configuração systemd | Prompt próprio de perfis | README; packaging |
| MQTT5/WS/WSS | Ausentes | Evolução conjunta com broker nas fases correspondentes | src/cli.rs; src/session.rs |

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
