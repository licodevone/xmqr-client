# C149 v1.1 — distribuir client 0.43.0 / bridge 0.5.0

Data: 2026-10-19. Entrada publicada client0.42.1/bridge0.4.0; C147 aceito,
C148 em validação final conforme promptv1.15. Broker0.43.0/admin0.11.0
têm versões independentes. Pedido autoriza concluir fase8, commits/push,
tags/releases nos marcos aceitos. Este prompt precede as mudanças de versão.

Objetivo: distribuir C147/C148 em uma minor experimental, sem modificar
tags antigas nem latest. Nenhuma implementação nova de protocolo nesta fatia.
Só promover a candidata após o aceite funcional C148; CI da revisão exata
da candidata continua obrigatória, mesmo com resultados da fonte anterior.

Escopo: Cargo/lock, versão da bridge, Dockerfile/CI/exportador/lifecycle,
documentação instalada no pacote (incluir bridge-mqtt5.md), README/VERSIONING/
CHANGELOG/nota de release, registro C149 e artefatos dist não versionados.
Atualizar gates de versão e ciclo descartável0.42.1→0.43.0→0.42.1→0.43.0,
conferindo SHA do pacote anterior contra a publicação C146. Sem instalação
no host do operador. Configuração preservada e serviços continuam opt-in.

Migração: v1 legado preservado; draftv2source5→dest311 agora exige política
explícita legacy_metadata_policy=drop. O novo modo ambasMQTT5 exige estado
formato2 em diretório novo, IDs novos e migração/rollback documentados após
drenar as rotas. Nunca apagar journal pendente para viabilizar downgrade.

Aceite: fmt/test/clippy/build/inventário, regressões e seis jobs CI completos;
C148 wire e brokers no debug e no ELF do pacote, crash no debug (hooks não
existem no release), versões/ELFs e instalação/rollback Ubuntu26amd64.
Imagem Bookwormlinux/amd64 não root10001:10001, SIGTERM, labels da revisão
exata; extrair ELF e executar C148 wire/interop nessa imagem, sem confundir
hooks debug com produto release. Preservar FAIL/NOT_RUN e revisar prompt
antes de correções. Sem certificação integral, cluster ou atomicidade distribuída.

Após gatesPASS: tag anotada imutávelv0.43.0, imagem numerada
lepinheiro100/xmqr-client:0.43.0, prerelease com seis anexos (deb,tar,manifesto
e respectivos sidecars). Baixar os seis anexos e confrontar hashes, conferir
tag/alvo remotos e pull/run da imagem por digest. Não publicar candidata
reprovada. Registrar evidências e reconciliar P235/T02/plano em prompt próprio;
fechar fase8 somente após comprovar cada requisito e distribuição final.

Revisão v1.1, antes de corrigir gate legado: auditoria estática detectou
verify_bridge_brokers.py exigindo prefixo0.4.0. Atualizar sua expectativa
para0.5.0, mantendo todos os cenários. Não executar a candidata com oráculo
da versão anterior nem alterar as expectativas de rollback0.42.1/0.4.0.
