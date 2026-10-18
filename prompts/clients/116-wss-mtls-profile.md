# C116 — perfil TOML para WSS mTLS em loopback

Entrada: client 0.33.0 publicado em `v0.33.0`, C114 aceita WSS mTLS local pela CLI; broker 0.40.0 distribuído. Git limpo após registro C115. Este prompt precede qualquer alteração funcional.

Objetivo: permitir `[connection].ws_url = "wss://127.0.0.1:PORTA/mqtt"` no perfil TOML v1 e `--ws-url` explícito com `--profile`, reutilizando CA/cert/key, usuário e arquivo privado de senha. `pub`/`sub` MQTT 3.1.1/5 continuam iguais; não criar estado durável nem migração. O perfil continua mTLS; WS `open-lab` por perfil não entra neste recorte.

Escopo: `src/profile.rs`, testes de parser/precedência e integração real com broker P212; README, matriz e registro. URL aceita somente IP loopback e mesmas recusas de C114. Se perfil ou ambiente trouxer `host`/`port` junto de URL efetiva, recusar conflito sem escolher destino silenciosamente. `tls_password`, laboratório, outbox e manifesto de assinatura permanecem incompatíveis com URL. URL não pode conter userinfo, query, fragmento ou senha. Não criar `MQTT_CLIENT_WS_URL` nesta fatia.

Aceite: perfil WSS pub/sub MQTT 3.1.1/5 contra broker mTLS real, precedência da flag explícita, recusa de URL inválida/host/port/perfil TLS-senha e segredo não vazado em erro; regressão de perfil TCP, WS laboratório sem perfil e C114 CLI. Fmt, suíte, Clippy, build, licenças/distribuição e CI da fonte exata. Registrar PASS/FAIL/BLOCKED; checklist apenas parcial da fase 7. Sem tag/release automática nesta fatia.
