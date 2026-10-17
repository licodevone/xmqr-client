# C83 — Topic Alias compacto no publisher MQTT 5

Entrada: C81 e broker P144 aceitos, releases 0.21.0/0.23.0 publicadas. Este prompt precede qualquer alteração funcional C83. A fase 6C global permanece aberta.

Objetivo: `pub --protocol-version 5 --repeat N --repeat-interval-ms MS --publish-topic-alias A` com N entre 2 e 1000 e A entre 1 e 16. Usar o Topic Alias Maximum anunciado pelo broker no CONNACK: falhar antes de qualquer PUBLISH se A exceder o valor anunciado ou se a propriedade estiver ausente. O primeiro PUBLISH envia o tópico completo e a propriedade Topic Alias; os seguintes, na mesma conexão, enviam tópico vazio e o mesmo alias, mantendo payload, propriedades, QoS, retained e serialização/ACK de C81. Não persistir associação após reconnect. Sem negociação, o comportamento existente permanece inalterado.

Exclusões: alias automático/múltiplos tópicos, stream/linha, reconnect/replay, alteração de MQTT 3.1.1, alteração do broker e alegação de conformidade MQTT 5 integral.

Aceite: CLI rejeita valores/combinações inválidos; testes wire conferem CONNACK ausente/0/menor que A, primeiro tópico completo, tópico vazio subsequente, QoS0/1/2 e propriedades preservadas; integração TCP real com XMQR e Mosquitto 2.0.22 em contêiner descartável; regressões C78/C80/C81, fmt, testes Rust, Clippy, build, licenças e CI. Atualizar docs e checklist somente após aceite. Sem ensaio físico ESP32 nem release isolada.

Fonte normativa: OASIS MQTT 5 §3.3.2.3.4 e §3.2.2.3.8, https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html.
