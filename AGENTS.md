# Instruções xmqr-client

Antes de toda alteração/melhoria, criar prompt em prompts/clients com ID/versão,
estado Git/local, objetivo, escopo, arquivos, critérios de aceite e validação.
Execução autorizada permite preparar prompt e executar; leitura não exige prompt.
Registro real em prompts/registros, com PASS/FAIL/BLOCKED separados.
Leia README, ORIGIN.json e skills pertinentes. Prompts históricos não são execução.

Cliente independente Rust2024/MSRV1.88, rumqttc; não importar o crate do broker.
Binário mqtt-client preservado. Broker e mqtt-admin ficam em projeto vizinho.
Preservar limite 1024 bytes UTF-8, payload4096, segurança TLS/SAN, senha oculta,
0600 Linux para arquivo secreto e loopback dos laboratórios. Sem --insecure.
Não registrar secrets; ACK não comprova sucesso de negócio. Sem recursos novos
por consequência da extração. Não instalar, commit/push/tag/release por padrão.

Gates: fmt --check, test --locked, clippy --all-targets -D warnings, build.
Integrações usam broker separado e --client explícito; não executar estado real.
Wsl indisponível é BLOCKED. Não remover fonte original até aceite da cópia.
Versão0.7.0 local herdada inicialmente, sem release; evolução independente
somente em prompt autorizado. Cada melhoria deve ser guiada pelos prompts.


## Checkpoint atual — systemd/Ubuntu26

P47 (broker0.10.0) / C31 (client0.5.0): integração systemd e pacote deb
Ubuntu26.04 amd64. Tags0.9/0.4 são publicadas e imutáveis. Seções anteriores
são históricas. Ver docs/ubuntu-systemd.md e registros de validação atuais.
Commit/push/prerelease desta etapa autorizados explicitamente pelo usuário.
