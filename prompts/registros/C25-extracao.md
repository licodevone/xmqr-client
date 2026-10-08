# Registro C25 - extração local, 2026-10-08

Estado: separação estrutural CONCLUÍDA, gates Windows PASS.
Integrações Unix ainda BLOCKED; não há aceite funcional completo P43/C25.
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

## Finalização estrutural autorizada - revisão C25 1.1

Após gates Windows e nova conferência de todos os hashes/backup, removidos
somente os cinco fontes duplicados; diretório vazio removido sem recursão.
Backup continua na workspace e fontes rastreados têm recuperação pelo Git.
Broker/admin em xmqr; cliente independente em xmqr-client. Rumqttc e suas
dependências exclusivas removidos do manifest/lock/inventário do broker.
Rpassword permanece para mqtt-admin. Import OpenOptions condicionado a Unix,
sem mudança de semântica; Clippy all-targets do broker agora PASS no Windows.
Build all-targets/check e bins broker/admin PASS. READMEs e guias de build,
laboratório, cliente e Will adaptados. Scripts de integração recebem --client
explícito e independente. Prompts históricos preservados como origem.

Integrações com estado durável, 1024 UTF-8/retained/Will, monitor HTTP/MQTT e
gates Unix permanecem bloqueados por WSL. Nenhuma durabilidade/teste enfraquecido.
Nenhum novo recurso de cliente, instalação, GitHub, commit/push/tag/release.
O mqtt-admin permanece no broker; P43 com aceite completo pendente.
