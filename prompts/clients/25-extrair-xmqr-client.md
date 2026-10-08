# C25 - Extrair xmqr-client, versão 1.0, 2026-10-08

Autorização: separar o cliente em D:/projects/my-project/xmqr-client; broker e
mqtt-admin permanecem em D:/projects/my-project/xmqr. P43 em andamento deve ser
preservado. Destino inexistente na inspeção; não sobrescrever destino existente.
Origem HEAD bc4cc55 mais alterações locais P42/P43. Cliente atualmente integra
pacote mqtt-broker 0.7.0 local; não há --version nem versão própria publicada.

Criar projeto Rust independente, nome xmqr-client e binário mqtt-client para
compatibilidade. Preservar inicialmente versão 0.7.0 herdada, sem release;
versionamento futuro independente guiado por prompts. Manter rumqttc/TLS/Will,
1024 bytes UTF-8, quotas, credenciais e UX existentes. Substituir apenas constante
importada do broker por contrato local explícito, testado nos dois projetos.

Antes de remover fonte original: cópia byte-a-byte verificada com hashes,
backup local e gates do novo projeto. Preservar MIT e atribuições, prompts C*,
registro de origem, README, AGENTS e skills próprias com prompts-primeiro.
Não implementar melhorias sugeridas, não criar GitHub/git/commit/tag/release.
Não instalar ferramentas ou reiniciar serviços.

Aceite: fmt/test/Clippy/build de ambos, inventários MIT, pub/sub QoS0/1/2,
1024 UTF-8/retained/Will com broker e cliente independente. Validar mesma cópia
Windows por /mnt/d em WSL; bloqueios declarados, sem conclusão fictícia.
Remoção do duplicado e ajustes finais do broker ficam pendentes caso integrações
Unix não sejam verificáveis. Registrar caminhos, versão e recuperação.

## Revisão 1.1 - finalização estrutural autorizada, 2026-10-08

Pedido atual: ater-se ao broker e cliente, avaliar finalização segura da separação
com build/testes Windows aprovados e backup, sem alegar integração validada.
Pode remover duplicado após reconferir SHA256 original/backup e gates do cliente,
preservando registro de recuperação. Integração Unix segue requisito de aceite
funcional P43 e C25, BLOCKED se WSL indisponível; separação de pastas não equivale
a esses gates. Atualizar imports/dependências/docs e inventário do broker,
mantendo mqtt-admin e P43; não alterar persistência ou implementar recursos.
Corrigir somente import Unix OpenOptions para permitir Clippy Windows, sem mudar
comportamento. Preservar prompts históricos e registrar versão local sem release.
