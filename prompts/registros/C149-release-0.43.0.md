# C149 — client0.43.0 / bridge0.5.0

Prompt [C149v1.0](../clients/149-release-0.43.0-bridge-0.5.0.md) publicado
em45ca641 antes das alterações de versão. C148 possui aceite local registrado;
CI da fonte45ca641 em execução. Esta candidata integra C147/C148 sem alterar
o core broker0.43.0/admin0.11.0. Sem nova tag/release publicada por preparação.

Gates da candidata, pacote/rollback, imagem, tag/release e conferência remota
estão pendentes. Fase8 só pode ser fechada após confrontar P235 com resultados
reais. Não se instala pacote no host nem se migra estado privado automaticamente.

## Preparação e gates locais

Promptv1.1 atualizou a expectativa de versão da fixture legada antes de rodar
o novo executável; v1.2 acrescentou negativos diretos após auditoriaP235.
Gates nativos0.43.0: fmt/test(208PASS)/clippy(-Dwarnings)/build/inventário e
sete contratos de distribuiçãoPASS. C139legado mTLS/QoS2 tambémPASS, incluindo
CA/SAN/senha/ACL/reconexão e seus dois checkpoints de crash.

Security-debug1 preserva8PASS e falha de oráculo: esperava diagnóstico interno
que o produto redige corretamente. Após revisãov1.3, a fixture prova a recusa
SUBACK0x87 por peer independente e estado/ausência de entrega, sem alterar core.
Security-debug2:11/11PASS, oito negativas CA/SAN/senha/certificado por perna,
ACLsourceSUBACK, ACLdestPUBLISH com Refused durável/zeroACK/reabertura recusada,
e burstQoS0 com journalSent preservado/nenhum segundo encaminhamento.
Relatórios em[docs/conformance/c149](../../docs/conformance/c149/).

A CI39300446226 da preparação d22a63a foi cancelada como superada pela
inclusão desses gates, sem alegar reprovação ou aceite integral. A CI funcional
C14839300031174 terminou seis jobsPASS; relatóriosdebug/pacote lidos e copiados
para o registroC148. A distribuição exige a próxima CI da revisão exata final.

## Correção de parada SIGTERM antes da publicação

A imagem f695eb6 passou wire33PASS/1NOT_RUN, brokers34PASS e segurança11PASS,
mas o ensaio SIGTERM mostrou exit-15. Diagnóstico separado confirmou journal
Sent byte a byte preservado/zeroACK, sem perda de obrigação; não se declarou
parada graciosa. RelatóriosPASS/FAIL originais preservados em
[image-before-sigterm-fix](../../docs/conformance/c149/image-before-sigterm-fix/).

Promptv1.4 precedeu a correção: o binário bridge reutiliza signals.rs e mantém
ouvinte global durante configuração e execução de todas as rotas. CLI/--check
e códigos de falha preservados. Não há dependência nova ou limpeza de journal.
Gates Rust208/fmt/clippy/build/inventárioPASS novamente. Security-debug3:
13/13PASS; SIGTERM antesCONNACK e com Sent pendente termina0 em menos de5s,
preserva bytes, retoma mesmosIDs/sessões/PID1DUP e só confirma origem37 após
o destino. ELFdebugSHA256
`dd340f07cad6169362694ed7c7ce7340894bf83ca4527500f846d92ba1d8583c`.

CI39301146015 da imagem anterior foi cancelada como superada após a descoberta,
sem alegar falha do Rust (jobPASS) ou aceite dos outros jobs cancelados. A
próxima revisão completa e sua imagem/pacote substituem essa preparação.
