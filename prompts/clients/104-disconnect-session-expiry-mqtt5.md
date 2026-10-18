# C104 — Session Expiry no DISCONNECT MQTT 5 do client

Estado de entrada: client independente 0.28.0/C102 publicado, árvore limpa; broker 0.35.0/P131 aceita Session Expiry no DISCONNECT. O client configura o intervalo no CONNECT, mas seu rumqttc fixado só enfileira DISCONNECT normal sem propriedades. Fase 6B global aberta.

Objetivo observável: em `pub --protocol-version 5` e `sub --protocol-version 5 --count N`, expor `--disconnect-session-expiry-seconds 0..4294967294|never` para alterar explicitamente o prazo da sessão no DISCONNECT normal. Quando omitida, preservar exatamente o pacote abreviado atual e usar o prazo do CONNECT. Rejeitar antes da rede: MQTT 3.1.1, `sub` sem `--count`, valores inválidos e valor não zero se o CONNECT anuncia expiração zero.

Fonte normativa: [OASIS MQTT 5.0](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html), §3.1.2.11.2 e §3.14.2.2.2. A propriedade `0x11` é única; ausente usa o intervalo do CONNECT. CONNECT zero proíbe DISCONNECT não zero. O broker já implementa a transição P131.

Escopo: CLI/uso, `src/v5_session.rs`, fork fixado `vendor/rumqttc-0.25.1` apenas na fila/serialização de DISCONNECT, unitários/wire, integração descartável com broker P131, documentação e registro. Não alterar MQTT 3.1.1, TLS, perfis, estado local, versão, admin ou broker. Preservar confirmação de saída após `Outgoing::Disconnect`, ordem UNSUBACK/DISCONNECT e ausência de segredos em logs.

Aceite: CLI positivos/negativos antes da rede; bytes MQTT 5 com propriedade única e caso omitido idêntico ao anterior; sessão persistente criada, encerrada com zero e reconexão sem Session Present, além de caso não zero preservado, com broker real; fmt, suíte completa, Clippy, build, inventário/licenças e CI. Checklist C104 apenas após gates. Sem tag/release isolada; 6B global segue aberta.
