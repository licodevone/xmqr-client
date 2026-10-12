# Imagem Docker do cliente MQTT

A imagem é independente do broker, construída a partir deste repositório e do
`Cargo.lock`. Rust 1.99.0 corresponde à validação Linux atual. A imagem final
Debian slim contém o `mqtt-client`, certificados públicos do sistema e os
avisos de licença; o processo roda como UID/GID 10001.

```sh
docker build -t xmqr-client:0.7.1 .
docker run --rm xmqr-client:0.7.1 --version
```

Passe o subcomando e seus argumentos depois do nome da imagem. TLS/mTLS usa
arquivos montados em modo somente leitura. O arquivo de senha, quando usado,
precisa ser regular, 0600 e pertencer ao UID do processo; monte arquivo seguro
fora da imagem e nunca passe a senha como argumento.

```sh
docker run --rm \
  -v "$PWD/certs:/run/certs:ro" \
  xmqr-client:0.7.1 sub \
  --host broker --port 8883 --topic 'sensors/#' --topic 'alerts/#' \
  --username device --password-file /run/certs/password \
  --ca /run/certs/ca.crt --cert /run/certs/client.crt --key /run/certs/client.key \
  --output jsonl
```

Para publicar dados binários, passe `-i` e `--stdin` ou monte um arquivo com
`--message-file`; `--line-mode` publica uma mensagem por linha. JSON Lines de
mensagens/conclusões continuam em stdout e logs em stderr. No Windows PowerShell,
use `-i` com um fluxo redirecionado ou um bind mount para o arquivo.

`--open-lab` só alcança um broker no loopback do mesmo namespace de rede. Não o
use como forma de conectar containers separados; para a rede entre containers,
use TLS/mTLS e confira o SAN do certificado contra o nome de host escolhido.
As imagens são distribuídas por arquivos anexados às releases. Os builds
baixam imagens-base/dependências; não houve publicação em registry nesta entrega.

## Distribuição P63/C40 — linux/amd64

As releases0.11.1/0.7.1 incluem imagem exportada tar.gz, checksum e pacote
.deb Ubuntu26amd64. A imagem DebianBookworm contém seu runtime; isso permite
executar em hosts Linux com Docker e arquitetura compatíveis, sem homologar
todas as distribuições ou arm64. Não há publicação em registry nesta entrega.

Após baixar arquivo e checksum na release, valide e carregue:

```sh
sha256sum -c xmqr-client-0.7.1-linux-amd64.tar.gz.sha256
docker load -i xmqr-client-0.7.1-linux-amd64.tar.gz
```

Secrets são externos e devem ser legíveis por UID10001, com chave/bundle/senha
0600 e diretório acessível. SAN precisa corresponder ao nome usado para conectar.
No teste a rede usa hostbroker com SAN DNS:broker. Não colocar chaves/estado
na imagem. Configure arquivos próprios antes de iniciar; fixtures não são para uso.

Use --read-only --cap-drop=ALL --security-opt no-new-privileges no run. O volume
do broker precisa ser gravável por10001. SIGTERM faz shutdown coordenado; use
`docker stop --time 20 xmqr-broker` antes de atualizar. Conserve volume/configuração
e backup offline; rollback da imagem não equivale a garantia de compatibilidade
de qualquer formato futuro. Imagens desta etapa preservam o estado existente.

Validação entre containers: mTLS/QoS1, retained após restart, shutdown0, CA e
senha erradas recusadas. Versão/ID/labels/proveniência registrados. Rootfsread-only
e caps removidas aprovados. Sem HEALTHCHECK superficial ou monitor externo.
