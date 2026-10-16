# C62 rev1 — encerramento de assinatura MQTT 5 com UNSUBACK

Entrada: `main` `84ddaff`, versão 0.14.0, árvore limpa. `sub --protocol-version 5 --count N` encerra com DISCONNECT assim que atinge a contagem, sem exercer UNSUBSCRIBE/UNSUBACK. O broker P121 passará a suportar essa operação.

Objetivo: após atingir `--count`, cancelar cada filtro MQTT 5 ativo, esperar UNSUBACK e conferir razão `Success` antes de enviar DISCONNECT. Tratar razão de recusa, quantidade incorreta de razões e timeout como erro, sem afirmar cancelamento não confirmado. MQTT 3.1.1 e assinatura contínua permanecem. Fonte: [OASIS MQTT 5 §§3.10–3.11](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html).

Escopo: `src/v5_session.rs`, testes wire, integração real, README/changelog e registro. A API rumqttc 0.25.1 envia um filtro por UNSUBSCRIBE. Sem novo comando CLI, sem sessão persistente/expiração, sem versão/instalação nova.

Aceite: fixture observa UNSUBSCRIBE 5 antes de DISCONNECT e devolve UNSUBACK; negativos de razão/timeout; integração com broker P121; fmt/test/clippy/build e CI. Registrar resultados reais antes de ticar o plano.

Rev2 após o primeiro Clippy: corrigir o padrão `Event::Incoming` para a forma aninhada exigida pelo lint e repetir a validação. O teste wire positivo passou.

Rev3 após a primeira integração: o status em modo texto sai no stdout, não no stderr. O harness seleciona saída `jsonl`, cujo status sai no stderr, e aguarda esse aviso antes de publicar; repetir a integração sem atribuir a falha à implementação MQTT.

Rev4: a saída `jsonl` codifica payload em `payload_bytes`; comparar os bytes decodificados no harness em vez de procurar texto literal. O comando de assinatura terminou com sucesso, inclusive UNSUBACK.
