# C81 rev2 — repetição sequencial de PUBLISH MQTT 5

Entrada: client C80 aceito e broker P144 aceito; a fase 6C global segue aberta. Este prompt precede qualquer alteração funcional C81.

Objetivo: permitir `pub --protocol-version 5 --repeat 2..1000 --repeat-interval-ms 1..60000` com as mesmas fontes únicas que já são válidas no MQTT 3.1.1 (`--message` ou `--message-file`). Publicar uma mensagem por vez, aguardar envio QoS0 ou PUBACK/PUBCOMP QoS1/2 antes da próxima, manter a conexão viva durante o intervalo e respeitar Receive Maximum/Maximum Packet Size do CONNACK via rumqttc 0.25.1. Reaplicar propriedades MQTT 5 em cada PUBLISH e emitir uma confirmação por publicação. Falha/timeout após enviar deve informar resultado desconhecido ou lote parcial sem replay/reconexão automática.

Exclusões: modo linha/stream, Topic Alias compacto, múltiplas mensagens em voo, persistência QoS local e novo transporte. Não alterar MQTT 3.1.1. Propriedades de MQTT 5 continuam limitadas como em C77/C78.

Aceite: testes CLI e wire de QoS0/1/2, ordem/contagem, propriedades, intervalo, erros de ACK/queda/timeout e nenhuma mensagem extra; TCP real com broker P144 e Mosquitto 2.0.22, retained/estado quando aplicável; regressões C78/C80; fmt/test/Clippy/build/licenças/CI. Checklist só após CI. Sem release isolada nem ensaio físico ESP32.

Fonte normativa: OASIS MQTT 5 §4.9 (Receive Maximum), https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html.

Revisão rev2 após integração local: Docker Desktop não expõe o daemon nesta distro WSL (`docker` é um stub), embora o Docker do Windows esteja ativo. Separar o ciclo do container do verificador Python: executar XMQR sempre; quando `C81_MOSQUITTO_PORT` for informado, validar também o Mosquitto descartável criado pelo host Windows. Registrar as duas execuções e o digest da imagem. Não reduzir os critérios de interop.
