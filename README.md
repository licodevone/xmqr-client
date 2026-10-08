# xmqr-client

Projeto Rust independente, extraído do cliente XMQR. Pacote `xmqr-client`,
binário compatível `mqtt-client`. Versão local inicial **0.7.0**, herdada do
manifest de origem; não é release publicada. Próximas versões são independentes.

```powershell
cd D:\projects\my-project\xmqr-client
cargo build --locked --bin mqtt-client
cargo run --locked -- --help
```

No WSL use `/mnt/d/projects/my-project/xmqr-client`. O broker permanece em
`D:/projects/my-project/xmqr`; mqtt-admin pertence ao broker.
Cliente usa rumqttc: pub/sub QoS0/1/2, retained, sessões e Last Will; TLS/mTLS
com validação de CA/SAN e credenciais. Laboratórios somente loopback.
Tópicos/filtros: até 1024 bytes UTF-8; payload até 4096. Sem novos recursos.
Senha interativa oculta; `--password-file` somente Linux/WSL, proprietário
atual e modo 0600. Não há persistência local de sessão ou reconexão nova.

Gates: `cargo fmt --all -- --check`, `cargo test --locked`,
`cargo clippy --locked --all-targets -- -D warnings`, `cargo build --locked`.
Integração: no broker, execute `scripts/verify_last_will.py --broker CAMINHO
--client CAMINHO_DO_CLIENTE_INDEPENDENTE` em ambiente Unix isolado.
Guia anterior preservado em docs/original-mqtt-client.md como referência:
caminhos de build e links relativos nele pertencem ao repositório original.
Prompts C19-C24 são históricos/propostas, não autorização de implementação.

Toda melhoria deve começar por um prompt com ID/versão, estado, objetivo,
escopo, arquivos, invariantes, aceite e testes. Veja AGENTS.md e C25.
MIT e atribuições originais preservadas. Separação estrutural concluída com gates Windows e backup verificado.
Integrações Unix com o broker permanecem pendentes; não há aceite funcional completo.
