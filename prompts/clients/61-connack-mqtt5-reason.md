# C61 rev1 — motivo de recusa de CONNECT MQTT 5

Entrada: `main` em `ec6af88`, versão 0.14.0, árvore limpa; C60 mostra razões de PUBACK/SUBACK, mas CONNECT recusado apresenta mensagem genérica. O broker P120 passará a retornar SUBACK ACL `0x87`.

Objetivo: mostrar o `ConnectReturnCode` recebido e seu código hexadecimal em CONNACK recusado, mantendo falha de transporte distinta e sem imprimir segredos. Confirmar na integração real o SUBACK ACL `NotAuthorized (0x87)` do broker P120. Fonte: [OASIS MQTT 5 §§3.2.2 e 3.9.3](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html).

Escopo: `src/v5_session.rs`, teste wire, script de integração, CHANGELOG, documentação/registro. Mapear os códigos aceitos por rumqttc 0.25.1 sem presumir que o discriminante Rust seja o byte de protocolo. Não alterar retry, autenticação, saída de sucesso, MQTT 3.1.1, estado, versão ou instalação WSL.

Aceite: fixture CONNACK `0x83` exibe enum e código; integração com broker real recebe SUBACK `0x87`; publicação permitida permanece funcional. Gates `fmt`, `test --locked`, `clippy --locked --all-targets -D warnings`, build e CI. Registrar resultados reais antes de ticar o plano.
