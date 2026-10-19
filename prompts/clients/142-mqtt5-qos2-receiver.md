# C142 v1.1 — receptor QoS 2 padrão

Revisão após FAIL de compilação do teste: `Outgoing` é exportado na raiz de
rumqttc, não em `v5`. Corrigir somente a importação do harness; nenhum GREEN
foi obtido nessa tentativa. O contrato funcional abaixo permanece igual.

v1.2: teste histórico `mqtt5_default_unknown_pubrel_remains_rejected_without_pubcomp`
codificava o defeito auditado. Substituir a expectativa por PUBCOMP 0x92 e
provar que a mesma conexão continua recebendo e encerra por --count; não
apenas apagar o teste nem esperar timeout ocioso de 90s.

v1.3: o teste adaptado completou PUBCOMP/UNSUBSCRIBE/DISCONNECT, mas sua
asserção final procurava texto 'x' em JSONL que contém bytes [120]. Corrigir
somente a verificação para o payload binário real, preservando o erro anterior.

Entrada/autoridade: C140/C141. Corrigir MQTT-4.3.3-10 e MQTT-4.3.3-11 no
client MQTT 5 comum. O remetente sintético exercita obrigações do receptor;
não constitui autorização normativa para retransmissão arbitrária do remetente.

Escopo: `vendor/rumqttc-0.25.1/src/v5/state.rs`, testes em `tests/`, fixture
wire autocontida em `scripts/`, CI e `XMQR-PATCHES.md`. Original Apache-2.0
preservado; código próprio MIT, sem mudança MQTT 3.1.1. Enquanto o PID estiver
aguardando PUBREL, novo PUBLISH QoS2 desse PID recebe PUBREC mas não nova entrega
à aplicação. Após PUBREL o PID pode representar nova transação. PUBREL repetido
ou desconhecido não zero recebe PUBCOMP 0x92; conhecido recebe sucesso. PID zero
continua inválido. Sem mapa ilimitado, sem alterar callback/eventos manuais que
o journal C129/C139 usa, e sem deadlock do eventloop por supressão de evento.

Aceite: RED publicado para as duas falhas; GREEN fixture e testes do estado via
API pública cobrindo ACK repetido, reuso do PID, dois PIDs, manual ACK/journal,
PID zero e recepção de erro PUBREL. Regressões bridge MQTT5 QoS0/1/2 e MQTT311,
outbox e journal. Wire fixture deve usar loopback/portas efêmeras, deadlines,
cleanup de filhos, payload fictício, relatórios exclusivos com hashes; CI não
pode depender do checkout vizinho. Não alterar relatório histórico P232 para
apagar FAIL. Registrar resultado atual separado e distinguir recomendações.

Fonte: [OASIS MQTT 5.0 OS](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html),
§4.3.3 e PUBCOMP §3.7.
