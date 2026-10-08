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
Tópicos/filtros: até 1024 bytes UTF-8; payload até 4096. Melhorias C26 abaixo.
Senha interativa oculta; `--password-file` somente Linux/WSL, proprietário
atual e modo 0600. Não há persistência local de sessão. Reconexão opcional descrita abaixo.

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


## Marco C26 (não publicado)

Versão do manifest preservada em **0.7.0**, herdada e local; a tag local existente
não foi reescrita. A decisão sobre a versão própria e publicação permanece pendente.

`--reconnect-attempts N` aceita 0..10 (padrão 0, falha sem reconectar).
Usa o mesmo EventLoop rumqttc, Client ID e opções de sessão. O orçamento é total
por processo, incluindo falhas antes do CONNACK e quedas após conectar, sem reset.
As esperas são 100, 200, 400, 800, 1600, 3200ms e no máximo 5s nas demais tentativas;
cada conexão mantém os timeouts da biblioteca e os deadlines do cliente.
TLS, recusa de autenticação e erros de protocolo não são retentados.

Em `sub`, sessão retomada preserva a assinatura; sessão ausente exige SUBSCRIBE
novamente. Use --client-id ID --clean-session false para solicitar sessão estável
entre conexões/processos; a retomada depende de o broker ainda ter essa sessão.
Não há persistência de estado MQTT no disco do cliente. --count continua contando
através das reconexões. Ctrl+C cancela poll/espera e tenta DISCONNECT por até 1s
se o transporte ainda estiver disponível, sem reconectar durante o encerramento.

Em `pub`, reconexão é permitida somente antes de enfileirar PUBLISH. Se a conexão
cair enquanto se aguarda o envio/ACK, o cliente falha com resultado desconhecido,
sem reenviar a publicação. Uma repetição manual pode produzir duplicatas.
QoS0 indica envio; PUBACK/PUBCOMP confirmam apenas o protocolo do broker,
não sucesso de negócio nem processamento pelo assinante.

`--output text|jsonl` mantém texto por padrão. JSON Lines escreve um objeto por
linha em stdout; estados operacionais, reconexões e erros vão para stderr.
Mensagens têm event, topic, qos, retain e payload_bytes (array de
inteiros 0..255, inclusive payload vazio), preservando binários sem conversão UTF-8.
Conclusão de pub usa event: "publish_complete" e confirmation igual a sent
(QoS0) ou broker_protocol_ack (QoS1/2). Credenciais não integram esses objetos.

```powershell
cargo run --locked -- sub --open-lab --topic 'test/#' --count 2 --client-id stable-id --clean-session false --reconnect-attempts 3 --output jsonl
```

Validação C26 usa fixtures MQTT próprias em loopback/portas efêmeras, sem executar
o broker vizinho ou Mosquitto. Isso não substitui os gates de integração Unix/TLS
com broker independente ainda pendentes em C25.

Limitação verificada: rumqttc 0.25.1 limpa o estado de recepção QoS2 ao perder
a conexão. Se a queda ocorrer entre PUBLISH recebido e conclusão PUBREL/PUBCOMP,
o cliente falha explicitamente, sem reconectar nessa fase. O handshake recebido
tem deadline fixo de 8s; `--count` aguarda sua conclusão antes de DISCONNECT.

Prompts: veja [índice e ordem](prompts/README.md) e [matriz de cobertura](prompts/COBERTURA.md).
Os históricos não constituem um bootstrap independente comprovado do zero.
