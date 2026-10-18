# Bridge XMQR experimental — C125/C126/C134/C136

O binário `mqtt-bridge` é um **sidecar experimental** do pacote do client. A release 0.39.0 contém bridge 0.1.0 com QoS 0/1; o client 0.40.0 distribui bridge 0.2.0 com QoS 2 C134. Por padrão, as duas pontas usam MQTT 3.1.1 sobre TLS/mTLS. Uma instância só encaminha da origem para o destino. `route.qos=0` mantém o modo melhor esforço C125. `route.qos=1` usa sessão persistente na origem, assinatura QoS 1, um encaminhamento em voo por vez e ACK manual da origem somente após o PUBACK do destino. O broker XMQR continua independente do processo da bridge.

Na fonte C136, `source_protocol="mqtt5"` na raiz do TOML seleciona origem MQTT 5 **somente com `route.qos=0`**; o destino permanece MQTT 3.1.1. A assinatura envia `Retain As Published=1`, permitindo encaminhar como retained tanto o replay inicial quanto publicações retained vivas, sem marcar publicações normais como retained. A bridge espera o destino conectar antes de assinar; se ele falhar, encerra sem afirmar que o replay foi entregue. A opção ainda não integra a release 0.40.0. QoS 1/2 continuam no modo MQTT 3.1.1 até contrato e testes próprios.

```toml
version = 1
source_protocol = "mqtt5"
max_retries = 8
# [source], [destination] e [route] seguem o exemplo abaixo; use route.qos = 0.
```

Exemplo de configuração privada (`/etc/xmqr/bridge.toml`); substitua os caminhos, portas e identidades pelos seus. Use usuários e certificados próprios para a bridge em cada broker e conceda somente os filtros/tópicos necessários na ACL.

```toml
version = 1
max_retries = 8
topology_file = "/etc/xmqr/bridge-topology.toml"
link_id = "origem-destino"

[source]
broker_id = "origem"
host = "broker-origem.exemplo"
port = 8883
client_id = "bridge-origem-01"
username = "bridge-origem"
password_file = "/etc/xmqr/bridge-source.password"
ca = "/etc/xmqr/ca.crt"
cert = "/etc/xmqr/bridge-source.crt"
key = "/etc/xmqr/bridge-source.key"

[destination]
broker_id = "destino"
host = "broker-destino.exemplo"
port = 8883
client_id = "bridge-destino-01"
username = "bridge-destino"
password_file = "/etc/xmqr/bridge-target.password"
ca = "/etc/xmqr/ca-destino.crt"
cert = "/etc/xmqr/bridge-target.crt"
key = "/etc/xmqr/bridge-target.key"

[route]
source_prefix = "sensors/"
target_prefix = "mirror/"
qos = 1
```

Para `qos=1` ou `qos=2`, crie também o manifesto **privado** `/etc/xmqr/bridge-topology.toml` (`chmod 0600`), compartilhado por todas as instâncias gerenciadas da topologia. A identidade de cada broker associa nome, host e porta; cada link declara a rota exata. O exemplo acima usa:

```toml
version = 1

[[brokers]]
id = "origem"
host = "broker-origem.exemplo"
port = 8883

[[brokers]]
id = "destino"
host = "broker-destino.exemplo"
port = 8883

[[links]]
id = "origem-destino"
source_broker = "origem"
destination_broker = "destino"
source_prefix = "sensors/"
target_prefix = "mirror/"
qos = 1
```

C127 recusa links com IDs duplicados, pontas declaradas com mesmo endereço resolvido, rota local divergente ou ciclo entre os links. Prefixos que se interceptam também contam como caminho possível de retorno. Para migrar uma configuração QoS 1 anterior à C127, adicione `topology_file`, `link_id` e `broker_id` em ambas as pontas. QoS 0 sem manifesto continua compatível; para validar sua topologia, forneça os mesmos campos e declare o link com `qos = 0`. A configuração e o manifesto são carregados apenas na partida: reinicie todas as instâncias com o mesmo manifesto validado quando alterar a topologia.

Na fonte C134, para QoS 2 altere `route.qos` e o `qos` do link no manifesto para `2` e acrescente `journal_dir = "/var/lib/xmqr-client-bridge-lab"` na raiz do arquivo de configuração. Com `xmqr-client-bridge@lab.service`, o systemd cria esse diretório privado com modo `0700` para `xmqr-client`; em execução manual, crie um diretório absoluto privado `0700` sob o mesmo usuário antes de iniciar. Cada instância precisa de diretório próprio. O journal guarda identificador de pacote, tópico e hash do payload, não o payload ou a senha. Não apague ou troque o diretório com pendências; se a sessão MQTT de origem desaparecer, a bridge recusa a retomada e conserva o journal para análise. A assinatura QoS 2 só confirma o PUBLISH na origem depois de receber PUBCOMP do destino e persistir a obrigação; PUBREL posterior conclui o journal. Uma queda entre PUBCOMP no destino e o registro local ainda pode causar duplicação entre os brokers.

O destino QoS 2 também mantém sessão MQTT persistente para retomar PUBREL/PUBCOMP após queda de rede ou reinício do broker. Configure um `destination.client_id` exclusivo por instância e preserve o estado do broker de destino durante a retomada. A bridge não mantém outbox local do destino entre reinícios do próprio processo; por isso, a janela de duplicação entre brokers permanece.

Com essa rota, `sensors/sala/temperatura` torna-se `mirror/sala/temperatura`. Remova `qos` ou use `qos=0` para o comportamento C125. A bridge recusa mesma ponta de origem/destino, IDs iguais, prefixos com curingas ou inválidos e caminhos relativos. Arquivos de configuração, senha e chave exigem permissões `0600` ou mais restritas no Linux; CA e certificado precisam ser arquivos regulares. `max_retries` aceita 1–20 e limita erros consecutivos de conexão em cada ponta. A bridge não exibe payload nem senhas nos logs; imprime estado de conexão e contadores de encaminhamento/descarte.

Para experimentar **a partir da fonte** no Ubuntu/WSL:

```sh
cd /mnt/d/projects/my-project/acmebr-projects/acmebr-broker-client/xmqr-client
cargo build --locked --bin mqtt-bridge
target/debug/mqtt-bridge --version
target/debug/mqtt-bridge --config /etc/xmqr/bridge.toml
```

Os testes de integração usam brokers e certificados descartáveis; execute `python3 scripts/verify_bridge_brokers.py --broker-root ../xmqr` e `python3 scripts/verify_bridge_mosquitto.py --broker-root ../xmqr` para QoS 0, e repita ambos com `--qos1` para QoS 1 e `--qos2` para QoS 2 da fonte C134; execute `python3 scripts/verify_bridge_topology.py --broker-root ../xmqr` para a cadeia real de duas bridges e três brokers, após construir `mqtt-broker`, `mqtt-admin`, `mqtt-client` e `mqtt-bridge` no WSL. A fixture Mosquitto usa Docker apenas para os containers Mosquitto 2.0.22 e os remove ao terminar. O ensaio QoS 1 com dois XMQR cobre retained já armazenado, atualização seguida de replay em nova assinatura e crash após PUBACK do destino, antes do ACK da origem, seguido de reentrega; usa um checkpoint disponível apenas em builds de depuração. Mensagens duplicadas nesse intervalo são esperadas. O ensaio QoS 2 cobre dois checkpoints de queda, antes e depois de PUBREC na origem; no build de release, `--skip-crash-checkpoint` verifica os demais cenários sem alegar cobertura dessas janelas.

**Limites:** QoS 0 é melhor esforço. Uma mensagem pode ser descartada durante indisponibilidade do destino, e uma publicação enfileirada não tem journal local nem confirmação de entrega final. Em QoS 1/2, PUBACK/PUBCOMP do destino confirmam a troca MQTT nessa perna, não entrega ao assinante, armazenamento externo nem transação exatamente uma vez entre brokers. No modo MQTT 3.1.1, uma publicação retained entregue ao assinante ativo normalmente chega com `retain=false`, portanto não há garantia de reproduzir como retained toda publicação viva; o modo MQTT 5 C136 resolve esse caso apenas para a rota QoS 0. Propriedades MQTT 5 da publicação não são encaminhadas ao destino. A bridge não é bidirecional. O manifesto impede ciclos **apenas dentro do conjunto completo de instâncias gerenciadas que usa o mesmo arquivo**; não carrega marcador de percurso por mensagem e não detecta bridges externas, configurações divergentes, mudanças DNS após a partida ou links omitidos. A checagem de aliases usa os endereços resolvidos na partida; IPv6 literal não é aceito nesta fatia. Recomeça a assinatura após reconexão, mas esgota o número configurado de falhas consecutivas. A fase 8/T02 permanece aberta até essas lacunas e a distribuição serem tratadas.
