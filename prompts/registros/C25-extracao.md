# Registro C25 - extração local, 2026-10-08

Estado: cliente independente criado e gates Windows PASS; separação definitiva
BLOCKED por integrações Unix. Broker mantém fonte original por segurança.
Prompt clients/25-extrair-xmqr-client.md criado antes das edições.

Broker D:/projects/my-project/xmqr; cliente D:/projects/my-project/xmqr-client.
Pacote novo xmqr-client, binário mqtt-client, versão0.7.0 local herdada da origem,
sem release. Não há --version. Próximas melhorias/versionamento independentes
seguem prompts-primeiro; sugestões de recursos não foram implementadas.

Cópia inicial dos cinco fontes e backup conferidos byte-a-byte/SHA256; ORIGIN.json
registra origem bc4cc55 + working tree0.7.0, hashes e backup na workspace.
Adaptado somente import de MAX_TOPIC_BYTES para contrato local1024 e cfg Linux
de constante de password-file, sem alteração de comportamento. Dependências
comuns do Cargo.lock preservadas nas versões de origem, sem upgrades incidentais.
LICENSE byte-a-byte idêntica, SHA256
ACFD4BB82DB195B2BA0A8A1EA6C5048D2E66C05A0FD0BD843E08D8A46D34B465.

README, AGENTS, skill própria, prompts C19-C25, manifest/lock, origem e inventário
de dependências presentes. Prompts antigos são referência, não execução.

Windows, cargo --locked --offline:12 testes PASS; Clippy all-targets -Dwarnings
PASS; build PASS; fmt --check PASS. Inventário MIT -Xutf8 --check PASS.
Logs c25-windows-{test,clippy,build}.log na workspace do executor.

WSL 0x8007274c/timeout12s: integrações1024UTF-8,retained,Will e gates Linux
BLOCKED. Script broker verify_last_will.py já aceita --client explícito e deverá
receber o executável independente. Nenhuma integração real com broker durável
foi aprovada nessa retomada. Nenhuma instalação/commit/push/tag/GitHub ocorreu.

Fonte src/bin/mqtt-client e dependências/fluxos antigos permanecem no broker
até aceite de integração. Portanto desmembramento definitivo não está concluído.
Após recuperar WSL: gates ambos, integração com --client novo, conferir backup,
remover somente duplicado autorizado, adaptar docs/build/CI afetados e revalidar.
O mqtt-admin permanece no broker. P43 preservado e também com aceite pendente.
