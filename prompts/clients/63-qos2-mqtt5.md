# C63 rev1 — publicação e assinatura QoS 2 MQTT 5

Entrada: `main` `9c6e77c`, versão 0.14.0, árvore limpa. `--protocol-version 5` rejeita QoS2; C62 já aguarda UNSUBACK ao concluir `sub --count`. Broker P122 anunciará Maximum QoS padrão 2 e aceitará ciclo QoS2.

Objetivo: `pub --protocol-version 5 --qos 2` aguarda PUBCOMP antes de indicar sucesso, mostra motivo de PUBREC/PUBCOMP recusado e não reenvia resultado desconhecido. `sub --protocol-version 5 --qos 2` aceita SUBACK QoS2 e confirma entrega inbound pelo fluxo rumqttc antes do UNSUBSCRIBE/DISCONNECT. Fonte: [OASIS MQTT5 §§3.3–3.7 e 4.3.3](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html).

Escopo: validação CLI, `v5_session.rs`, fixtures wire, integração real, README/changelog/registro. Preservar modo 3.1.1, limites, autenticação, Clean Start e demais restrições MQTT5 atuais. Sem novas opções, estado/migração, versão ou instalação WSL.

Aceite: pub/sub QoS2 completos, recusa PUBREC `0x87`, timeout/sem sucesso prematuro, integração com P122 e gates fmt/test/clippy/build/CI. O client usa rumqttc 0.25.1; verificar eventos reais de PUBREC/PUBCOMP e tratamento de entrega.

Rev2 após fixture de assinatura: o formato de saída texto mostra bytes do payload, como no teste C58, não o texto literal. O ciclo PUBREC/PUBREL/PUBCOMP e UNSUBACK passou; ajustar a expectativa de saída e repetir.

Rev3 após integração própria: acrescentar interop real MQTT5 QoS2 com `mosquitto_pub` e `mosquitto_sub` 2.0.22 nas duas direções no harness isolado, usando somente credenciais de teste; repetir o ensaio e registrar separadamente seu resultado.

Rev4 após primeiro ensaio externo: `mosquitto_sub` encerrou antes do SUBACK. Acrescentar diagnóstico limitado ao stderr de teste, identificar se o CONNECT envia propriedades fora do subconjunto P122 e decidir se cabe um vetor suportado ou se a interop deve ficar marcada como bloqueada; não ampliar o protocolo implicitamente.

Rev5: diagnóstico apontou `Client Identifier not valid`: o broker exige ID explícito neste contrato, enquanto Mosquitto gerou ID vazio. Fixar IDs de teste distintos com `-i` nos comandos externos e repetir, sem alterar o broker.

Rev6: com ID explícito, o harness não reconheceu a linha de SUBACK do Mosquitto no prazo. Expor o texto de depuração acumulado no erro de timeout para separar falha do broker de expectativa incorreta do harness; ajustar apenas após inspecionar esse resultado.

Rev7: stderr permaneceu vazio com `-d`; a detecção de prontidão do harness externo não é confiável nesta instalação. Usar uma curta janela fixa de inicialização em loopback e conferir o resultado final/saída do assinante, sem atribuir PASS antes de receber a mensagem.
