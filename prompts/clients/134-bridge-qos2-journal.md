# C134 v1.1 — bridge QoS 2 com journal da origem

## Entrada e fonte normativa

`main` limpo em `19cc0e9`, client 0.39.0 publicado. C125–C127 e C133 dão
bridge MQTT 3.1.1 unidirecional TLS/mTLS QoS 0/1, topologia gerenciada e
pacote opt-in; fase 8/T02 segue aberta. No fork rumqttc atual, ACK manual de
PUBLISH QoS 2 envia PUBREC, mas `MqttState::clean` esquece IDs de entrada
após desconexão e um processo novo também não os restaura. Contrato:
[OASIS MQTT 3.1.1 §4.3.3 e §4.4](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html).

## Objetivo e escopo

Aceitar `route.qos=2` com sessão persistente na origem, assinatura QoS 2,
publicação QoS 2 no destino e confirmação PUBREC da origem **após** PUBCOMP
do destino. Gravar em journal privado o vínculo `source client ID`, broker,
rota, Packet Identifier, tópico, payload digest e fase antes de PUBREC.
Após restart, restaurar a fase de recebimento somente com Session Present=1
e identidade/rota idênticas; recusar estado órfão/corrompido/divergente sem
conectar nem descartá-lo. Concluir journal somente após PUBREL/PUBCOMP da
origem. Um único encaminhamento em voo por instância, no máximo 32 IDs
aguardando PUBREL, limite de bytes e arquivo, lock exclusivo e gravação
atômica com fsync. A origem pode emitir mais de um PUBLISH antes dos
respectivos PUBREL; o journal não pode sobrescrever obrigação anterior.

Arquivos previstos: `src/bin/mqtt-bridge.rs`, API mínima do fork vendorizado
`vendor/rumqttc-0.25.1/src/{eventloop,state}.rs`, fixture de dois brokers e
Mosquitto, docs da bridge e testes pertinentes. TOML v1 mantém QoS 0/1 sem
migração; `journal_dir` absoluto/privado é obrigatório somente em QoS 2.
Não incrementar versão ou publicar release até gate próprio de distribuição.

## Invariantes e limites

- QoS 2 é o contrato de cada perna MQTT. Entre brokers, crash após PUBCOMP
  do destino mas antes do commit local ainda pode duplicar publicação; não
  prometer transação distribuída ou entrega exatamente uma vez fim a fim.
- Não enviar PUBREC da origem se o destino não confirmou PUBCOMP. Timeout,
  ACL, falha TLS/SAN, quota ou corrupção preservam obrigação/journal.
- A fase pós-PUBREC deve responder a PUBREL retomado sem republicar no
  destino. PUBLISH DUP do mesmo Packet Identifier antes de PUBREL não deve
  criar segunda obrigação; payload/tópico divergente é falha fechada.
- Session Present=0 com journal pendente não apaga nem força replay; recusa
  controlada com instrução de recuperação. IDs de sessão são exclusivos.
- Preserve regressões QoS 0/1, limites de payload, ACL, mTLS e ciclo de
  topologia. Não embutir segredos/payload em logs ou evidências.

## Aceite

Testes unitários para journal/identidade/limites/corrupção, ACK e restauração
do fork. Fixture wire/TLS de dois brokers: QoS 2 normal, reconexão, retained,
negativos e duas quedas determinísticas (antes de PUBREC e após PUBREC antes
de PUBREL). Interop Mosquitto nas duas direções. `cargo fmt --check`, `cargo
test --locked`, `cargo clippy --all-targets -D warnings`, build, CI da fonte
exata e registro em `prompts/registros/C134-bridge-qos2-journal.md` com
PASS/FAIL/BLOCKED separados. Só riscar a fatia C134 após prova; T02 e fase 8
globais continuam abertos enquanto retained vivo, loops externos e demais
critérios não forem aceitos.
