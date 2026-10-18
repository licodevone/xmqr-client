# Bridge XMQR experimental — primeira fatia C125

O binário `mqtt-bridge` é um **sidecar em `main` após v0.36.0**, ainda não incluído no `.deb`/Docker da release 0.36.0. Ele usa MQTT 3.1.1 sobre TLS/mTLS nas duas pontas, assina `source_prefix + #` com QoS 0 e publica no `target_prefix` com QoS 0, sem retained. Uma instância só encaminha da origem para o destino. O broker XMQR continua independente do processo da bridge.

Exemplo de configuração privada (`/etc/xmqr/bridge.toml`); substitua os caminhos, portas e identidades pelos seus. Use usuários e certificados próprios para a bridge em cada broker e conceda somente os filtros/tópicos necessários na ACL.

```toml
version = 1
max_retries = 8

[source]
host = "broker-origem.exemplo"
port = 8883
client_id = "bridge-origem-01"
username = "bridge-origem"
password_file = "/etc/xmqr/bridge-source.password"
ca = "/etc/xmqr/ca.crt"
cert = "/etc/xmqr/bridge-source.crt"
key = "/etc/xmqr/bridge-source.key"

[destination]
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
```

Com essa rota, `sensors/sala/temperatura` torna-se `mirror/sala/temperatura`. A bridge recusa mesma ponta de origem/destino, IDs iguais, prefixos com curingas ou inválidos e caminhos relativos. Arquivos de configuração, senha e chave exigem permissões `0600` ou mais restritas no Linux; CA e certificado precisam ser arquivos regulares. `max_retries` aceita 1–20 e limita erros consecutivos de conexão em cada ponta. A bridge não exibe payload nem senhas nos logs; imprime estado de conexão e contadores de encaminhamento/descarte.

Para experimentar **a partir da fonte** no Ubuntu/WSL:

```sh
cd /mnt/d/projects/my-project/acmebr-projects/acmebr-broker-client/xmqr-client
cargo build --locked --bin mqtt-bridge
target/debug/mqtt-bridge --version
target/debug/mqtt-bridge --config /etc/xmqr/bridge.toml
```

Os testes de integração usam brokers e certificados descartáveis; execute `python3 scripts/verify_bridge_brokers.py --broker-root ../xmqr` e `python3 scripts/verify_bridge_mosquitto.py --broker-root ../xmqr` após construir `mqtt-broker`, `mqtt-admin`, `mqtt-client` e `mqtt-bridge` no WSL. A segunda fixture usa Docker apenas para os containers Mosquitto 2.0.22 e os remove ao terminar.

**Limites:** QoS 0 é melhor esforço. Uma mensagem pode ser descartada durante indisponibilidade do destino, e uma publicação enfileirada não tem journal local nem confirmação de entrega final. A bridge não transporta retained, QoS 1/2 ou propriedades MQTT 5, não é bidirecional e não evita loops criados por outras bridges. Recomeça a assinatura após reconexão, mas esgota o número configurado de falhas consecutivas. Não aponte duas bridges em direções opostas sem uma política externa de prevenção de loops. A fase 8/T02 permanece aberta até essas lacunas e a distribuição serem tratadas.
