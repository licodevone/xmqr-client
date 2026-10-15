# C50 rev1 — C05: perfis completos e precedência de ambiente

2026-10-15. Entrada commit/tag `47266aa687bc40d6d73b43e892268424eca9406b`/
`v0.11.1`, árvore limpa. C47/C48 aceitos; broker P100/P101 integrado localmente.
Pedido do mantenedor: finalizar a fase 4 com prompt antes da funcionalidade,
marcação do plano e publicação quando apropriado.

Objetivo: concluir C05 sobre `--profile` TOML v1. Adicionar campos reutilizáveis
de `clean_session`, `reconnect_attempts` e `total_timeout_seconds`, com tipos
estritos. Para perfis, aplicar precedência **flag CLI explícita > variável de
ambiente explícita > arquivo > default**. Variáveis permitidas:
`MQTT_CLIENT_HOST`, `MQTT_CLIENT_PORT`, `MQTT_CLIENT_USERNAME`,
`MQTT_CLIENT_PASSWORD_FILE`, `MQTT_CLIENT_ID`, `MQTT_CLIENT_OUTPUT`,
`MQTT_CLIENT_QOS`, `MQTT_CLIENT_KEEPALIVE_SECONDS`,
`MQTT_CLIENT_TIMEOUT_SECONDS`, `MQTT_CLIENT_TOTAL_TIMEOUT_SECONDS`,
`MQTT_CLIENT_RECONNECT_ATTEMPTS`, `MQTT_CLIENT_CLEAN_SESSION`, além das
existentes `MQTT_CA_CERT`, `MQTT_DEVICE_CERT`, `MQTT_DEVICE_KEY`.

Variável inválida/não Unicode deve falhar sem revelar conteúdo nem cair para o
arquivo. Caminhos do TOML continuam relativos à pasta do perfil; caminhos de
ambiente e flags continuam relativos ao cwd. Arquivo de senha mantém verificação
0600/dono no Linux, limite e leitura apenas após parse. Não aceitar senha
inline, modo laboratório no perfil, certificado cliente em TLS-senha, payload,
Will ou shell. Sem `--profile`, CLI existente e seus fallbacks de certificados
continuam como hoje; não mudar wire, estado, QoS nem limites.

Aceite: unidade com ambiente injetado, negativos de precedência/schema/permissão
e integração real com broker mTLS/TLS-senha. Gates fmt/test/Clippy/build e
inventário; documentação/matriz/registro atualizados. Versão/tag/release em
marco próprio após gates, sem modificar versões do broker ou admin independente.

## rev2 — organização para lint

Unidades passaram, mas Clippy apontou função de expansão longa e dois padrões
de `if/else`. Extrair a coleta de overrides em helper sem mudar precedência,
ajustar os testes e repetir os gates.

## rev3 — integração de precedência

Estender laboratórios temporários P101 (mTLS) e P97 (TLS-senha) com perfil
TOML que contém host/porta/CA errados, ambiente que corrige porta/CA e flag
CLI que corrige host. Arquivo de senha continua privado e relativo ao perfil.
Exigir publicação QoS1 real antes de declarar C05 aceita.
