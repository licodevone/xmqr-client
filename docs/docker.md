# Imagem Docker do cliente MQTT

## Distribuição atual 0.14.0 — MQTT 5 inicial experimental

Imagem numerada `lepinheiro100/xmqr-client:0.14.0` publicada com fonte,
digest e hashes registrados em [C59](../prompts/registros/C59-release-0.14.0.md).
A [prerelease](https://github.com/licodevone/xmqr-client/releases/tag/v0.14.0)
contém o export e checksums. Nenhum `latest` foi movido.

## Histórico 0.9.0 — robustez da sessão

Imagem publicada e conferida por manifest, pull/run por digest, versão real
e source revision `e688c75a3779803e8f1868792dcc5e7a36f2dd6e`.
MQTT3.1.1 e robustez C45; MQTT5 não implementado.

```sh
docker pull lepinheiro100/xmqr-client:0.9.0
docker run --rm --network none --read-only --cap-drop=ALL lepinheiro100/xmqr-client:0.9.0 --version
```

Digest OCI publicado:
`sha256:e1c56183bd98fbe33c9fade58f81d61b2f60eb960c3704747528ec7f471a11ed`.
A [prerelease0.9.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.9.0)
inclui .debUbuntu26amd64, export Docker tar.gz, manifest/checksums e evidências.
Confira SHA antes de `docker load -i ARQUIVO.tar.gz`; o export não é pacote
de ELF nativo. Runtime da imagem Bookworm, independente do .debUbuntu.

Rebuild do checkout atual usa somente tag LOCAL ainda inexistente, nunca
substitui uma tag publicada. Revision de HEAD documental não é a fonte e688
atribuída ao artefato publicado.

```sh
docker build --platform linux/amd64 --build-arg VERSION=0.9.0 --build-arg VCS_REF="$(git rev-parse HEAD)" -t xmqr-client:0.9.0-local .
```

UID10001, credenciais/estado externos, CA/SAN e segredo privado preservados.
Limites e resultados em [C46](../prompts/registros/C46-release-fase3.md).
Comandos de build das seções históricas abaixo não servem no HEAD atual
e não devem sobrepor tags publicadas.

## Histórico do marco 0.8.1 — Docker Hub

Imagem Linux/amd64 no namespace autorizado, mantendo os filtros retained da
0.8.0. Nenhuma opção MQTT, credencial, limite ou dependência foi alterada.
Comandos abaixo pressupõem o aceite/publicação; conferir a tag/digest e
o manifesto da prerelease v0.8.1 antes de usar.

```sh
docker pull lepinheiro100/xmqr-client:0.8.1
docker run --rm --network none --read-only --cap-drop=ALL lepinheiro100/xmqr-client:0.8.1 --version
docker build --platform linux/amd64 --build-arg VERSION=0.8.1 --build-arg VCS_REF="$(git rev-parse HEAD)" -t lepinheiro100/xmqr-client:0.8.1 .
```

O build verifica a versão real do binário, corrigindo o label antigo 0.7.1.
O contexto allowlist exclui arquivos locais, snapshots, fixtures e chaves.
UID10001, SIGTERM, CA/SAN e secret-file privado continuam obrigatórios.
Use a imagem completa do Hub nos exemplos de configuração abaixo.

O .deb é Ubuntu26 amd64; o runtime da imagem é DebianBookworm com compilação
própria. Não instalar o .deb Ubuntu no Bookworm. .deb/checksums, imagem
tar.gz e manifesto de proveniência acompanham a prerelease. A exportação
scripts/release_assets.py exige fontes commitadas e labels/version/revision
coerentes, sem publicar automaticamente. Tags antigas/latest não são movidas.

Veja o [plano detalhado de paridade](https://github.com/licodevone/xmqr/blob/main/docs/plano-paridade-mosquitto.md).
As seções C40 abaixo são histórico, não indicam ausência de publicação atual.

## Histórico de distribuição local C40

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
