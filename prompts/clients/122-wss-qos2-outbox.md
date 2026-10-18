# C122 — outbox MQTT 5 QoS 2 por WSS mTLS

Entrada: client 0.35.0 publicado com outbox QoS 1 por WSS mTLS e retomada C120; outbox QoS 2 TCP C107 já possui estágios `Prepared`, `PublishMayHaveSent` e `PubRelReady`. O parser ainda recusa QoS 2 em URL.

Objetivo: permitir uma publicação MQTT 5 QoS 2 com outbox privada por WSS mTLS local/remoto, reutilizando a identidade de URL canônica da C119 e os estágios duráveis C107. `PUBREC` só avança para `PUBREL` após persistência do estágio; `PUBCOMP` conclui a operação. Em queda após PUBLISH, retransmitir mesmo Packet ID com DUP=1 se Session Present=1; após PUBREL, reenviar PUBREL. Sem sessão, falhar sem reenvio e preservar journal. WS aberto, assinatura persistente por URL e múltiplas mensagens continuam fora do escopo.

Aceite: parser positivo/negativo, fixture WSS mTLS para fluxo completo, PUBLISH perdido, PUBREL perdido, Session Present=0 e razões negativas, integração com broker real QoS 2/ACL, regressões QoS 1/TCP/assinatura, suíte/fmt/Clippy/licenças/CI da fonte exata. Documentar a garantia exata e limites. Sem tag/release isolada.
