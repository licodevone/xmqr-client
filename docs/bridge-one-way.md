# Bridge XMQR experimental — C125/C126

O binário `mqtt-bridge 0.1.0` é um **sidecar experimental** incluído no candidato `.deb`/imagem do client 0.37.0; a release 0.36.0 anterior não o contém. Ele usa MQTT 3.1.1 sobre TLS/mTLS nas duas pontas. Uma instância só encaminha da origem para o destino. `route.qos=0` (padrão) mantém o modo melhor esforço C125. `route.qos=1` usa sessão persistente na origem, assinatura QoS 1, um encaminhamento em voo por vez e ACK manual da origem somente após o PUBACK do destino. O broker XMQR continua independente do processo da bridge.

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

Para `qos=1`, crie também o manifesto **privado** `/etc/xmqr/bridge-topology.toml` (`chmod 0600`), compartilhado por todas as instâncias gerenciadas da topologia. A identidade de cada broker associa nome, host e porta; cada link declara a rota exata. O exemplo acima usa:

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

Com essa rota, `sensors/sala/temperatura` torna-se `mirror/sala/temperatura`. Remova `qos` ou use `qos=0` para o comportamento C125. A bridge recusa mesma ponta de origem/destino, IDs iguais, prefixos com curingas ou inválidos e caminhos relativos. Arquivos de configuração, senha e chave exigem permissões `0600` ou mais restritas no Linux; CA e certificado precisam ser arquivos regulares. `max_retries` aceita 1–20 e limita erros consecutivos de conexão em cada ponta. A bridge não exibe payload nem senhas nos logs; imprime estado de conexão e contadores de encaminhamento/descarte.

Para experimentar **a partir da fonte** no Ubuntu/WSL:

```sh
cd /mnt/d/projects/my-project/acmebr-projects/acmebr-broker-client/xmqr-client
cargo build --locked --bin mqtt-bridge
target/debug/mqtt-bridge --version
target/debug/mqtt-bridge --config /etc/xmqr/bridge.toml
```

Os testes de integração usam brokers e certificados descartáveis; execute `python3 scripts/verify_bridge_brokers.py --broker-root ../xmqr` e `python3 scripts/verify_bridge_mosquitto.py --broker-root ../xmqr` para QoS 0, e repita ambos com `--qos1` para QoS 1; execute `python3 scripts/verify_bridge_topology.py --broker-root ../xmqr` para a cadeia real de duas bridges e três brokers, após construir `mqtt-broker`, `mqtt-admin`, `mqtt-client` e `mqtt-bridge` no WSL. A fixture Mosquitto usa Docker apenas para os containers Mosquitto 2.0.22 e os remove ao terminar. O ensaio QoS 1 com dois XMQR cobre retained já armazenado, atualização seguida de replay em nova assinatura e crash após PUBACK do destino, antes do ACK da origem, seguido de reentrega; usa um checkpoint disponível apenas em builds de depuração. Mensagens duplicadas nesse intervalo são esperadas.

**Limites:** QoS 0 é melhor esforço. Uma mensagem pode ser descartada durante indisponibilidade do destino, e uma publicação enfileirada não tem journal local nem confirmação de entrega final. Em QoS 1, o PUBACK do destino confirma aceitação MQTT, não entrega ao assinante ou armazenamento externo. O sinalizador retained recebido da origem é repassado em QoS 1; no MQTT 3.1.1 uma publicação retained entregue ao assinante ativo normalmente chega com `retain=false`, portanto não há garantia de reproduzir como retained toda publicação viva. A bridge não suporta QoS 2 nem propriedades MQTT 5 e não é bidirecional. O manifesto impede ciclos **apenas dentro do conjunto completo de instâncias gerenciadas que usa o mesmo arquivo**; não carrega marcador de percurso por mensagem e não detecta bridges externas, configurações divergentes, mudanças DNS após a partida ou links omitidos. A checagem de aliases usa os endereços resolvidos na partida; IPv6 literal não é aceito nesta fatia. Recomeça a assinatura após reconexão, mas esgota o número configurado de falhas consecutivas. A fase 8/T02 permanece aberta até essas lacunas e a distribuição serem tratadas.
