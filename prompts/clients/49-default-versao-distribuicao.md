# C49 rev1 — corrigir default da imagem após C48

2026-10-15. Entrada `59bcc3403e94874b2193783f27961b229ad3e962`,
tag `v0.11.0` já enviada, árvore limpa. O teste de distribuição falhou porque
`ARG VERSION=0.10.0` diverge do Cargo0.11.0. Nenhuma release0.11.0 será
publicada. O pedido autoriza distribuição da fase4.

Objetivo: corrigir Dockerfile e emitir patch0.11.1 em novo commit/tag, sem
reescrever v0.11.0. Escopo: Cargo.toml/lock, Dockerfile, changelog e registro.
Não alterar comportamento MQTT/TLS. Validar gates, integração, contrato de
distribuição, imagem por versão/revisão e checksums antes da release0.11.1.
