# C128 — pacote, serviço e imagem da bridge experimental

Entrada: [P226](../../../xmqr/prompts/broker/226-bridge-package-release-contract.md). Este prompt precede qualquer mudança de pacote, Docker ou versão.

Incluir `mqtt-bridge` junto de `mqtt-client` no `.deb` Ubuntu 26.04 amd64 e na imagem OCI. Criar `xmqr-client-bridge@.service` sem enable/start automático; configuração por instância em arquivo 0600 do usuário de serviço, sem embutir segredo. `--version` deve informar bridge e versão da release. Atualizar docs de instalação, upgrade/rollback, direitos de arquivo, limites C125–C127 e versão. Provar pacote e imagem com dois brokers reais em TLS/mTLS QoS 1, sem alterar instalações WSL existentes; verificar regressões de client e bridge, CI, seis ou mais hashes remotos e release/tag somente após gates completos.

Ainda aberto após a release: proteção universal de loops MQTT 3.1.1, QoS 2, retained vivo, APT assinado e demais fases do plano. Não marcar T02 completo apenas pela distribuição experimental.
