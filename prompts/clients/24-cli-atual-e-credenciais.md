# C24 — Auditar o cliente Rust atual e credenciais

Tipo: manutenção existente. Leia src/bin/mqtt-client/{cli,credentials,main,session,tls}.rs,
docs/mqtt-client.md e contrato-base. Rumqttc é dependência atual: não reescrever
cliente com codec próprio por instrução histórica de clients/20.

## Prompt de trabalho

Audite pub/sub, --qos 0|1|2, --retain true|false, --clean-session true|false,
--count, --client-id, host/port e parâmetros PEM com o USAGE real. Confirme
restrições de Client ID e tópico/filtro (pendências P37/P38).
Preserve TLS com CA, SAN e certificado/chave; --open-lab anônimo e
--plain-auth-lab para senha/ACL sem TLS, ambos loopback. Não oferecer --insecure.
Audite --username, senha interativa e --password-file, permissões/rejeições reais,
precedência MQTT_CA_CERT/MQTT_DEVICE_CERT/MQTT_DEVICE_KEY, buffers e timeouts.
Não expor senha na linha de comando; não mudar saída de payload esperada pelo
usuário sem pedido. Não prometer armazenamento persistente do cliente, replay
de estado de processo ou reconexão ilimitada só por existir CleanSession no broker.

## Aceite futuro

Testes de CLI válida/inválida, flags incompatíveis, credencial ausente/privada,
modo loopback, CA/SAN inválidos, deadline, QoS/ACK, retained e sessão com ID
estável. Gate externo conforme P36; toda indisponibilidade registrada como BLOCKED.
Atualizar documentação apenas sob escopo autorizado; não implementar consumidor.
