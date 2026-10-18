# C114 — WSS mTLS local no client

Entrada: client 0.32.0 publicado, C112 aceitou WS local sem TLS; broker P212 em validação em `main` após 0.39.0. Git limpo em `0a26411`. Este prompt precede toda alteração funcional.

Objetivo: permitir `--ws-url wss://127.0.0.1:PORTA/mqtt` (ou `[::1]`) em `pub`/`sub` MQTT 3.1.1 e 5, no modo seguro mTLS padrão, usando a mesma CA, SAN, certificado cliente e chave do TCP seguro. O fork rumqttc 0.25.1 já oferece `Transport::wss_with_config` e envia/verifica `Sec-WebSocket-Protocol: mqtt`. WS `ws://` continua exclusivo de `--open-lab`.

Escopo: `src/cli.rs`, `src/session.rs`, `src/v5_session.rs`, testes de parser e integração com broker P212 real, documentação e registro. WSS somente em IP loopback neste recorte; sem `--host`/`--port` com URL, userinfo, query/fragmento, `--tls-password`, perfil ou outbox/manifesto durável. Credenciais MQTT continuam por prompt ou `--password-file` privado; nenhuma senha na URL. Não relaxar TLS/CA/SAN, não oferecer `--insecure` ou fallback WS.

Aceite: pub/sub WSS 3.1.1/5 e cruzamento com TCP no broker P212 com mTLS/ACL; recusa de CA/SAN/certificado/senha/ACL inválidos, URL/mode/destino conflitantes e resposta sem subprotocolo; regressão WS local e TCP/TLS. Fmt, suíte, Clippy, build, inventário/distribuição e CI da fonte exata. Sem estado durável novo ou migração. Marcar entrega parcial no plano após evidência, mantendo fase 7 global aberta. Sem mudar versão, tag ou release nesta fatia.
