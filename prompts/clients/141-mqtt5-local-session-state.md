# C141 v1.1 — Session Present exige estado local

Revisão após FAIL Clippy: run_inner passou de 100 linhas. Extrair a derivação
dos flags do estado durável para helper testável, sem silenciar a regra nem
alterar o contrato de retomada.

v1.2: regressão histórica de CONNECT (never/finito) esperava sucesso com
Session Present=1 sem estado local. Adaptar seu controle positivo para
Session Present=0; o novo negativo wire conserva a detecção de 1. Não remover
checagens de flags/expiração.

v1.3: após FAIL de compatibilidade C108, preservar a limpeza existente apenas
do manifesto recém-criado/nunca ativado quando Session Present inesperado é
recusado. Não deixar essa preparação mudar o resultado de uma segunda tentativa
sem estado anterior. Manifesto existente e journals continuam preservados.
Atualizar diagnóstico do harness para o novo erro estático. Após FAIL Clippy
ref_option, usar Option<&T> no helper. Inventário inicial ficou BLOCKED por
PATH sem cargo, não por licença; repetir com PATH explícito.

Entrada: mesma base/autoridade C140. Executar depois da proteção de capacidades.
Corrigir MQTT-3.2.2-4: processo novo sem Session State não aceita CONNACK de
sucesso com Session Present=1, ainda que Clean Start=false. Encerrar antes de
SUBSCRIBE/PUBLISH, sem reconexão automática ou descarte silencioso da sessão.

Escopo: `src/v5_session.rs`, testes wire/unitários e README. Estado é o journal
ou manifesto efetivamente carregado e validado, não apenas a presença da opção
de diretório. Preservar as validações existentes para primeira criação,
Prepared/MayHaveSent/AwaitPubComp, manifesto incerto/ativo e sessão perdida.
Não alterar formato durável, identidade, 0600, nem regravar estado do operador.
Sem estado e Session Present=0 permanece permitido; retomada durável válida
continua funcionando. Atualizar orientação histórica de retomada manual.

Aceite: RED publicado, GREEN pub/sub sem estado, Session Present=0 positivo,
primeiro diretório e retomadas duráveis existentes (QoS1/2 e assinatura).
Regressões e registro conjunto C140–C142 conforme C140; falha real exige revisão
antes do ajuste. Sem afirmação de processamento de negócio exatamente uma vez.

Fonte: [OASIS MQTT 5.0 OS](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html),
§3.2.2.1 Session Present.
