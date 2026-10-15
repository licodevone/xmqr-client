# C55 rev1 — Will binário por arquivo (C08)

2026-10-15. Entrada: client0.12.0 publicado; C52/C53/C54 implementados e
validados localmente. Will atual usa `--will-message` UTF-8; payload PUBLISH
aceita bytes por arquivo e limite4096. Não há MQTT5 neste client.

Objetivo: `--will-message-file PATH` como fonte alternativa a
`--will-message TEXTO`. Ler arquivo regular uma vez, com limite de 4096 bytes,
antes de abrir a rede. Preservar bytes NUL e UTF-8 inválido sem conversão; Will
vazio é válido. Nunca incluir path ou conteúdo em diagnóstico. `--will-topic`,
QoS e retain seguem as validações existentes; fontes simultâneas e opção sem
tópico são recusadas antes da conexão.

Escopo: parser, fonte de payload, sessão MQTT, testes wire/CLI, docs e registro.
Preservar comportamento de Will texto, quotas, CONNECT MQTT3.1.1, TLS e senha.
Nenhuma propriedade MQTT5 ou alteração no broker. Testar arquivo 4096/4097,
diretório/ausente, exclusividade, CONNECT com bytes binários e regressão texto.
Gates fmt/test/Clippy/build/inventário Ubuntu26. Não marcar fase 5 inteira.
