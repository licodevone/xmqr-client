# Imagem Docker do cliente MQTT

A imagem é independente do broker, construída a partir deste repositório e do
`Cargo.lock`. Rust 1.99.0 corresponde à validação Linux atual. A imagem final
Debian slim contém o `mqtt-client`, certificados públicos do sistema e os
avisos de licença; o processo roda como UID/GID 10001.

```sh
docker build -t xmqr-client:0.5.0 .
docker run --rm xmqr-client:0.5.0 --version
```

Passe o subcomando e seus argumentos depois do nome da imagem. TLS/mTLS usa
arquivos montados em modo somente leitura. O arquivo de senha, quando usado,
precisa ser regular, 0600 e pertencer ao UID do processo; monte arquivo seguro
fora da imagem e nunca passe a senha como argumento.

```sh
docker run --rm \
  -v "$PWD/certs:/run/certs:ro" \
  xmqr-client:0.5.0 sub \
  --host broker --port 8883 --topic 'sensors/#' --topic 'alerts/#' \
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
Essas imagens permanecem locais: nenhum registry é acessado e nenhuma tag Git é
criada por esta tarefa.
