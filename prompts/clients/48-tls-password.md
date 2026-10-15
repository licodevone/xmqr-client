# C48 rev1 — B10/C05: client TLS com senha sem certificado

2026-10-15. Entrada `4444c8086796c86974f644c177c4ad0ff6598961`,
`v0.10.0`, árvore limpa e prerelease publicada. Pedido do mantenedor: concluir
a fase 4, criando prompt antes de funcionalidade.

Objetivo: `--tls-password` explícito para servidor TLS com usuário/senha e ACL,
sem identidade X.509 do client. Perfil TOML v1 pode selecionar esse modo.
`--ca` ou `MQTT_CA_CERT` segue obrigatório; TLS verifica SAN e validade.

Escopo: parser, expansão de perfil, configuração rustls, testes e guia. Rejeitar
`--cert`/`--key` e as variáveis de identidade implícitas nesse modo; modos de
laboratório permanecem loopback e mTLS continua padrão. Senha só por prompt
oculto ou arquivo 0600 em Linux. Limites e reconexão existentes não mudam.

Validação: fmt/test/Clippy/build em Ubuntu 26, negativos de modo misto,
cert/key, CA ausente e SAN errada, integração com broker P97 para autenticação,
ACL e QoS. Documentar precedência CLI/perfil/env, resultados e limitações.
Compatibilidade: opt-in, sem alteração no wire MQTT ou formato de perfil.
