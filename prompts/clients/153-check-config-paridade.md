# C153 v1.1 — validar configuração de pub/sub e revisar paridade

Entrada: Client 0.43.0, bridge 0.5.0, fonte `a6d36f7d2fc86b8c45f240c700d8a313bd213417`.
Checkout limpo antes da tarefa. A bridge já oferece validação offline própria.

Objetivo: `mqtt-client --check-config pub|sub ...` usa a mesma análise de
argumentos/perfil do fluxo normal e valida materiais TLS e arquivo de senha
quando indicado. O fluxo pode aceitar senha interativa como requisito futuro
sem solicitá-la no check. Não lê payload/stdin nem abre conexão. Saída
redigida informa protocolo, operação e se a senha será interativa.

Escopo: `src/main.rs`, testes dirigidos, README, matriz Mosquitto e registro.
Comparar `mosquitto_pub`, `mosquitto_sub` e `mosquitto_rr`; registrar opções
equivalentes, diferentes e ausentes. Request/response e taxa são fatias próprias.

Compatibilidade: sem alterações em TLS/SAN, limites, transporte, QoS, perfis
ou formato de estado. Senhas/erros de arquivos não saem no diagnóstico.
Aceite: CLI válida e inválida, perfil TOML, credencial ausente/segredo
redigido, validação sem leitura de stdin nem rede; fmt/test/clippy/build locked.
Tag de checkpoint após gates no commit exato; release depende do pacote.

Revisão 1.1: a primeira fixture usou perfil em `open-lab`, combinação
recusada pelo parser existente. Preservar essa regra. Testar sucesso em
loopback com MQTT 5 explícito e que um perfil TLS válido sintaticamente
chega à verificação de materiais. A compilação direta com rlibs locais
permite testes dirigidos enquanto Cargo está bloqueado pelo cache.
