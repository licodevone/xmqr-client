# Bridge MQTT 5 durável — fonte C148

Este guia descreve o modo opt-in com MQTT 5 nas duas pontas, múltiplas rotas,
metadados de aplicação, prazo de validade e journal privado de encaminhamento.
O contrato e seus limites estão no [prompt C148](../prompts/clients/148-bridge-mqtt5-duravel-percurso.md).
O [guia anterior](bridge-one-way.md) continua descrevendo os modos com destino
MQTT 3.1.1 e seus journals próprios.

Na fonte C148, drafts v2 de origem MQTT 5 com destino 3.1.1 precisam declarar
`legacy_metadata_policy="drop"`, assumindo explicitamente o descarte dos
metadados de aplicação nessa fronteira. Esse campo é recusado com origem
3.1.1 e no modo MQTT 5 nas duas pontas, que preserva os metadados descritos
neste guia. Configurações v1 publicadas mantêm seu comportamento histórico.

## Disponibilidade e versão

A candidata **client 0.43.0 / bridge 0.5.0**, preparada por C149, incorpora
C147/C148. **A publicação ainda está pendente** dos gates da revisão exata,
do pacote e da imagem, além da conferência remota de tag e anexos.
A última release confirmada é **client 0.42.1 / bridge 0.4.0**, cujo pacote
e imagem não contêm C147/C148. Use o executável compilado da revisão candidata
registrada, com seu hash e evidências; `--version` confirma a versão declarada.
Consulte a [nota 0.43.0](releases/0.43.0.md) e o
[registro C149](../prompts/registros/C149-release-0.43.0.md) para o estado da distribuição.

O aceite é experimental e finito. C148 não declara certificação MQTT 5 integral,
produção, cluster ou commit atômico distribuído entre brokers. Uma publicação
remota cujo resultado ficou incerto pode ser repetida após queda, causando
duplicação entre as duas pernas. A aplicação consumidora continua responsável
por sua identificação de negócio quando precisar deduplicar operações.

## Como o encaminhamento funciona

Cada rota tem duas conexões MQTT independentes: uma assina o broker de origem
e a outra publica no broker de destino. Ambas usam TLS/mTLS, usuário, arquivo
privado de senha e validação do servidor. As permissões MQTT continuam sob
controle de cada broker.

O journal guarda a mensagem, seus metadados, a identidade e as fases da
transferência. Os commits locais precedem os ACKs correspondentes. Um ACK MQTT
confirma a responsabilidade daquela perna; o processamento pelo dispositivo ou
pela aplicação de destino não é confirmado.

Existe **um encaminhamento pendente por rota**. As identidades MQTT recebem o
sufixo `-ID_DA_ROTA`, dentro do teto de 128 bytes, e cada rota tem seu diretório
de estado. Uma falha fatal de um worker encerra o grupo de rotas.

## Preparar os arquivos privados

Escolha um diretório novo e vazio para este modo. O exemplo usa
`/var/lib/xmqr-bridge/v5-example`, pertencente ao usuário que executará a bridge.
Em um teste manual, este comando prepara o diretório para o usuário atual:

```bash
sudo install -d -m 0700 -o "$(id -un)" -g "$(id -gn)" \
  /var/lib/xmqr-bridge/v5-example
```

Se usar um serviço, atribua o diretório ao usuário do serviço. Prepare também
os arquivos do exemplo em `/etc/xmqr/bridge`, com caminhos absolutos reais:

- `mqtt5.toml` e `topology-v5.toml`, privados, com modo 0600.
- Um arquivo de senha por ponta, com modo 0600 e a senha MQTT correspondente.
- CA, certificado cliente e chave por ponta; a chave deve ser privada, modo 0600.
- Arquivos e diretórios reais: symlinks são recusados nas fronteiras validadas.

O TOML não expande `$HOME` ou variáveis de ambiente dentro de caminhos.
Os arquivos abaixo são exemplos de configuração; adapte portas, certificados,
usuários e concessões nos brokers antes de conectar.

## Configuração completa de duas rotas

Arquivo `/etc/xmqr/bridge/mqtt5.toml`:

```toml
version = 2
source_protocol = "mqtt5"
destination_protocol = "mqtt5"
bridge_id = "ponte-laboratorio"
max_hops = 4
response_topic_policy = "map-prefix"
state_dir = "/var/lib/xmqr-bridge/v5-example"
topology_file = "/etc/xmqr/bridge/topology-v5.toml"
max_retries = 8

[source]
broker_id = "origem"
host = "127.0.0.1"
port = 8883
client_id = "bridge-origem"
username = "bridge-source"
password_file = "/etc/xmqr/bridge/source.password"
ca = "/etc/xmqr/bridge/source-ca.crt"
cert = "/etc/xmqr/bridge/source-client.crt"
key = "/etc/xmqr/bridge/source-client.key"

[destination]
broker_id = "destino"
host = "127.0.0.1"
port = 8884
client_id = "bridge-destino"
username = "bridge-destination"
password_file = "/etc/xmqr/bridge/destination.password"
ca = "/etc/xmqr/bridge/destination-ca.crt"
cert = "/etc/xmqr/bridge/destination-client.crt"
key = "/etc/xmqr/bridge/destination-client.key"

[[routes]]
id = "temperatura"
source_prefix = "sensors/"
target_prefix = "mirror/"
qos = 1

[[routes]]
id = "umidade"
source_prefix = "metrics/"
target_prefix = "archive/"
qos = 2
```

Arquivo `/etc/xmqr/bridge/topology-v5.toml`:

```toml
version = 1

[[brokers]]
id = "origem"
host = "127.0.0.1"
port = 8883

[[brokers]]
id = "destino"
host = "127.0.0.1"
port = 8884

[[links]]
id = "temperatura"
source_broker = "origem"
destination_broker = "destino"
source_prefix = "sensors/"
target_prefix = "mirror/"
qos = 1

[[links]]
id = "umidade"
source_broker = "origem"
destination_broker = "destino"
source_prefix = "metrics/"
target_prefix = "archive/"
qos = 2
```

O formato aceita **uma a oito rotas**. IDs de rota são únicos, de até 32 bytes;
`.` e `..` são recusados. Prefixos terminam em `/`, não começam com `$` e os
prefixos de origem não podem se sobrepor. Os links do manifesto devem coincidir
com as rotas. Não misture `[[routes]]` com os campos do formato v1 `[route]`,
`link_id` ou `journal_dir`.

No destino MQTT 5, origem MQTT 5, `state_dir`, `bridge_id`, `max_hops` e política
de resposta são obrigatórios, inclusive em QoS 0. O ID da bridge é estável,
ASCII, de até 24 bytes, e deve identificar a instância entre as pontes do
percurso. Todas as suas rotas compartilham esse ID. `max_hops` aceita 1 a 8;
`max_retries` aceita 1 a 20.

## Compilar, conferir e conectar

Execute no Linux/WSL, a partir da cópia correta do repositório `xmqr-client`:

```bash
git rev-parse HEAD
cargo build --locked --release --bin mqtt-bridge
BRIDGE_BIN="${CARGO_TARGET_DIR:-target}/release/mqtt-bridge"
"$BRIDGE_BIN" --version
"$BRIDGE_BIN" --check --config /etc/xmqr/bridge/mqtt5.toml
```

`git rev-parse` identifica a revisão; o build gera o executável dessa fonte.
`BRIDGE_BIN` considera o diretório de build configurado no ambiente. O
diagnóstico `--check` valida formato, limites, privacidade, materiais TLS e
manifesto sem abrir conexões ou criar o estado das rotas. Ele exige que o
diretório pai de estado já exista. Senha aceita, ACL e disponibilidade precisam
da conexão real.

Depois de o diagnóstico terminar com sucesso, conecte explicitamente:

```bash
"$BRIDGE_BIN" --config /etc/xmqr/bridge/mqtt5.toml
```

Cada assinatura só é anunciada ativa depois do SUBACK. Ctrl+C encerra a
instância e preserva o estado. SIGTERM, usado por Docker/systemd, também
encerra normalmente com código zero e preserva a transferência pendente para
retomada; não confirma uma mensagem apenas para parar. O cancelamento abrange
configuração, conexão, disco e execução das rotas. Falhas de conexão têm retentativas limitadas;
recusas de protocolo, capacidades incompatíveis e erros de estado falham
preservando a obrigação. Verifique o estado das transferências antes de
trocar identidades, configuração ou versão.

## QoS, retenção e metadados

O destino usa o QoS configurado na rota. A origem pode entregar QoS menor:
origem 0 numa rota 2 continua sem confirmação na primeira perna; origem 1 numa
rota 2 recebe PUBACK depois da conclusão correspondente no destino. Elevar o
QoS da republicação não recupera garantias que faltavam na origem.

A cota Receive Maximum limita o fluxo QoS 1/2 de origem a um voo por rota.
QoS 0 não usa esse controle de fluxo. Um burst QoS 0 enquanto existe um voo
pendente pode exceder a cota e encerrar o grupo; mensagens QoS 0 ainda não
registradas podem se perder. Ajuste o publicador e o QoS conforme a necessidade.

A bridge preserva o bit retained recebido com Retain As Published: isso inclui
replay inicial, publicação retida ao vivo e conteúdo vazio retido, que remove
o valor retido no broker de destino. Uma nova assinatura solicita retained;
substituir uma assinatura existente não força um novo replay completo.

São preservados PFI, Content Type, Correlation Data binária e User Properties
ordenadas, inclusive chaves repetidas. PFI=1 exige payload UTF-8 válido. Topic
Alias e Subscription Identifier pertencem à conexão e não são propagados como
metadados de aplicação.

`response_topic_policy` define o tratamento do Response Topic:

| Política | Comportamento |
| --- | --- |
| `preserve` | Mantém o nome recebido, inclusive quando está fora do prefixo da rota. |
| `map-prefix` | Troca o prefixo da rota também na resposta; recusa Response Topic fora do prefixo de origem ou cujo resultado exceda o limite. |

Na rota `sensors/` → `mirror/`, uma resposta `sensors/reply` vira `mirror/reply`
com `map-prefix`. Essa política não cria uma rota de retorno: a aplicação deve
preparar o percurso e as permissões das respostas.

| Limite local | Valor |
| --- | --- |
| Nome de tópico e Response Topic | 1.024 bytes UTF-8, inclusive após remapeamento. |
| Content Type e Correlation Data | 256 bytes por campo. |
| Chave e valor de User Property | 256 bytes em cada campo. |
| User Properties | 16 no total, incluindo o marcador de percurso. Sem marcador recebido, restam até 15 propriedades de aplicação. |
| Payload | Teto interno de 64 KiB; o tamanho do pacote completo e seus metadados podem reduzir o valor efetivamente aceito. |
| Journal por rota | Até 2 MiB, um voo pendente e até 32 recibos de conclusão QoS 2. |

Os limites dos brokers também se aplicam. A bridge verifica Maximum QoS,
Retain Available e Maximum Packet Size do destino antes de transmitir;
incompatibilidades não causam redução silenciosa do QoS ou remoção do retained.

## Validade e contagem de descartes

Message Expiry vira um prazo absoluto privado. A bridge desconta o tempo de
fila e recuperação e recalcula o intervalo após os commits locais, antes do
envio. O TTL não é renovado. A quantização é conservadora: restante inferior
a um segundo é descartado antes de uma nova entrega, assim como expiry zero.

Se um PUBLISH QoS 1/2 anterior ficou incerto e o prazo terminou, a recuperação
completa a responsabilidade MQTT com PID estável, DUP e expiry zero, conforme
a fase. Depois de PUBREC no destino, retoma PUBREL sem republicar o payload.
Essa conclusão de protocolo não anuncia uma nova entrega válida da aplicação.

O journal mantém `discarded_expired` e `discarded_path`. O incremento durável
ocorre no mesmo commit que conclui o descarte, antes da confirmação da origem;
replay não conta novamente. Overflow de contador falha sem confirmar a origem.
Relógio regressivo em relação ao piso registrado causa recusa com estado
preservado. A precisão do relógio do sistema continua sendo responsabilidade
operacional.

## Percurso e prevenção de ciclos

A propriedade reservada `xmqr.bridge.path` contém uma lista de IDs separados
por vírgula. Existe no máximo uma ocorrência, com IDs válidos e únicos, até
oito entradas. A bridge acrescenta seu ID e mantém a ordem das propriedades de
aplicação. Marcador inválido causa erro sem ACK de sucesso.

Uma mensagem que já passou por esta bridge ou atingiu `max_hops` é descartada
duravelmente, contada e confirmada na origem conforme seu QoS. O manifesto
acíclico continua obrigatório como outra defesa, também nos modos legados.

A proteção entre pontes externas depende de cooperação: o intermediário deve
preservar a propriedade de percurso. Se ele apagar ou reconstruir o marcador,
essa informação é perdida e não há garantia de detectar o retorno. MQTT 3.1.1
não transporta essa propriedade; misturar uma perna 3.1.1 não oferece a mesma
proteção por mensagem.

## Journal e recusas conhecidas

O processo cria `state_dir/ID_DA_ROTA/` em modo 0700. Dentro dele,
`bridge-forward.json` e `bridge-forward.lock` exigem modo exato 0600. O arquivo
tem formato **2**, checksum e identidade vinculada a endpoints, client IDs,
usuários, mapeamento, QoS e política de percurso/resposta. Um lock exclusivo
impede duas instâncias de usar o mesmo estado.

| Fase persistida | Significado |
| --- | --- |
| `prepared` | Origem e mensagem registradas antes de iniciar o encaminhamento. |
| `sent` | PUBLISH pode ter sido enviado; recuperação trata o resultado como incerto. |
| `rel` | PUBREC de destino persistido; o próximo passo é PUBREL. |
| `done` | Transferência ou descarte concluído localmente; resta a confirmação pertinente na origem. |
| `refused` | ACK negativo conhecido do destino foi persistido; a rota está bloqueada para intervenção. |

PUBACK/PUBREC/PUBCOMP negativos conhecidos não confirmam a origem. O journal
fica em `refused`; reabrir a bridge falha antes de conectar, preservando a
obrigação. Uma recusa conhecida não autoriza retransmitir automaticamente a
publicação como se o resultado apenas tivesse ficado incerto.

Nesta fatia não existe comando de desbloqueio desse journal. Preserve seus
arquivos privados, configurações e identidades; investigue a recusa nos brokers
e planeje a intervenção offline considerando as responsabilidades das duas
pernas. Apagar o journal, zerar o diretório ou trocar IDs para esconder a recusa
pode perder uma obrigação e não faz parte do procedimento de recuperação.

As identidades de sessão aceitas são persistidas por perna. Uma sessão remota
anterior sem identidade local conhecida, ou uma sessão necessária que não
existe mais, provoca recusa. Estado corrompido, incompatível ou de outra
identidade é preservado e recusado.

Recibos de conclusão QoS 2 permitem responder PUBREL repetido após recuperação.
Enquanto a publicação está pendente, repetições não geram outro encaminhamento.
Depois de PUBCOMP, um PUBLISH com o mesmo PID é uma nova mensagem, inclusive
se vier com DUP=1 ou conteúdo igual. Esse é o contrato da
[seção 4.3.3 do MQTT 5](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html).

## Migração e rollback

1. Drene e pare a rota antiga, preservando sua configuração, estado e versões.
2. Prepare um diretório novo vazio para C148 e client IDs novos nas duas pontas.
3. Defina um ID estável da nova bridge, valide com `--check` e conecte.
4. Confirme os dois lados e a aplicação antes de aposentar a configuração anterior.

Os modos antigos continuam com seus journals próprios, como
`bridge-qos2.json`. O modo C148 recusa um diretório que contenha esse journal
legado. O formato experimental 1 de versões de desenvolvimento de C148 também
é recusado; não há migração implícita ou exclusão automática.

Para voltar ao engine anterior, drene o modo novo, pare-o e preserve seu
diretório de formato 2. Restaure a configuração e o estado antigos preservados,
verificando as sessões exigidas. A release publicada 0.42.1 não lê o formato 2.
Uma rota bloqueada ou com obrigação pendente precisa de análise offline antes
do rollback; trocar apenas o executável não converte nem conclui sua transferência.

## Verificações disponíveis

A fixture independente [verify_bridge_v5_wire.py](../scripts/verify_bridge_v5_wire.py)
aceita `--bridge`, `--broker-root` e um `--report` novo. Ela usa peers MQTT 5 raw
com mTLS, certificados descartáveis e estado próprio. A execução local final
registrou **33 PASS e 1 NOT_RUN**. O NOT_RUN é explícito: alias e Subscription
Identifier não foram negociados, portanto não foram injetados como tráfego
legal; a transformação desses campos é coberta por unitário de Publish real.

Os scripts de [crash/recuperação](../scripts/verify_bridge_v5_crash.py) e de
[integração entre brokers](../scripts/verify_bridge_v5_brokers.py) complementam
esse recorte. Registros de fontes, hashes, CI, interop e distribuição devem
ser consultados no marco correspondente. Resultados locais não substituem
aceite de pacotes, imagens ou fechamento global da fase 8.
