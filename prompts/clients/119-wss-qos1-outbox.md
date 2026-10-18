# C119 — outbox QoS 1 MQTT 5 por WSS mTLS

Entrada: client 0.34.0 publicado; C105 outbox QoS 1 local funciona sobre TCP, C114/C117 WSS mTLS local/remoto funciona sem estado local, broker 0.41.0 publicado. Este prompt precede o código.

Objetivo: permitir `pub --protocol-version 5 --qos 1 --outbox-dir` em URL `wss://` segura, preservando Clean Start false, Session Expiry positivo, Client ID fixo, journal privado, confirmação PUBACK e falha fechada quando Session Present é negado. WS aberto não pode receber estado durável.

Identidade: gravar a URL WSS canônica completa no campo `host` já existente do documento de outbox, sem alterar esquema. URL distinta e transporte TCP nunca reutilizam a mesma operação pendente; comparação de operação é exata e o journal fica intacto após recusa. Certificado/CA/username continuam validados no transporte; a URL não carrega segredo. Dados TCP existentes e rollback 0.34.0 mantêm compatibilidade.

Aceite desta fatia: parser positivo local/remoto (remoto exige `--allow-remote-wss`), negativos WS aberto/QoS2/assinatura/URL ambígua, isolamento de URL e TCP, pub/sub real com broker WSS mTLS e ACL, verificação de journal após PUBACK, regressões TCP/QoS2/sub e suite/fmt/Clippy/distribuição/licenças/CI da fonte exata. Recuperação de PUBACK perdido por WSS exige ensaio separado antes de afirmar durabilidade completa ou fechar fase 7. Sem tag/release isolada.
