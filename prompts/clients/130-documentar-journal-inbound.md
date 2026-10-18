# C130 v1.0 — guia do journal recebido e matriz de paridade

2026-10-18, antes das alterações documentais. Entrada main235f20d com C129
integrado; árvore C128/bridge/distribuição alheia permanece dirty e intocável.
Revisão independente C129v1.12 está corrigindo identidade dos SubscriptionIDs
e recusa de journal órfão em clone isolado. Não apresentar pendência comoPASS.

Criar docs/mqtt5-inbound-journal.md e atualizar docs/mosquitto-parity.md,
sem editar README/Cargo/CI/Docker/bridge existentes. Explicar opt-in
--subscription-state-dir, fases antes deACK, identidade/SP1,32records/64KiB,
diretório privado e conteúdo sensível, limites stdout→commit, countincompatível,
cancel/reconexão e diferença entre sessão do broker/journal do client.
Separar suporte limitado de aceiteglobal; nenhum manual promete transação
de negócio, exactlyonce de stdout, powerloss ou suporteWindows a journals.

Vincular registro/evidências reais e atualizar detalhes de formatoversão
somente após integração dos testesv1.12. Exemplos apenasloopback-lab explícitos,
com paths sintéticos; não solicitar nem inserir credenciais. Conferir flags
no parser, links locais e diffcheck. Documentação sozinha não muda versão/tag.
