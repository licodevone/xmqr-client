# C146 v1.0 — distribuir C144 no client 0.42.1

Data: 2026-10-19. Entrada: `main` `b36b9d6` após C145; runtime C144
`25c2e8a` com CI `39194165930` PASS. Publicados client 0.42.0/bridge 0.4.0,
broker 0.43.0 e admin 0.11.0. C144 ainda não está nos binários publicados.
O pedido contínuo autoriza commits/push/tags/releases nos marcos aceitos.

Objetivo: publicar prerelease corretiva client 0.42.1 com rejeição de NUL em
tópicos/User Properties, VBI redundante e flags reservadas inválidas MQTT 5.
Bridge mantém versão funcional 0.4.0, recompilada no pacote do client.
Sem nova funcionalidade, migração de estado ou alteração das tags anteriores.

Escopo: Cargo/lock, Dockerfile/CI, lifecycle descartável 0.42.0→0.42.1→0.42.0→0.42.1,
README/VERSIONING, guia e nota de release, registro C146 e artefatos em dist.
Hashes da release 0.42.0 devem ser conferidos antes de usar o pacote antigo.
O diretório de build local preservado por C145 segue ignorado e fora da imagem.

Aceite: fonte exata/CI com fmt, testes Rust, Clippy, build, inventário, C140–C142
42 casos, C144 14 casos, journals/WSS e bridge/regressões. Reaproveitar relatórios
da CI em vez de repetir campanhas iguais sem motivo. `.deb` Ubuntu26 amd64,
ELFs/versões corretos, instalação/rollback descartável preservando configuração
e unit opt-in; C144 em ELF do pacote. Imagem Bookworm não root/labels da revisão
exata e versão correta; C144 em ELF da imagem. Nenhuma instalação no host.

Só após gates PASS, publicar tag anotada imutável v0.42.1, imagem numerada sem
mover latest e prerelease com pacote, tar, manifesto e três sidecars SHA-256.
Baixar os seis anexos e confrontar hashes; confirmar digest remoto e execução
da imagem por digest. Preservar FAIL/BLOCKED e revisar prompt antes de correções.

Atualizar documentação/checklist do broker por prompt próprio depois de verificar
a publicação. A auditoria P232 histórica permanece imutável: um acompanhamento
novo deve separar as cinco falhas corrigidas na 0.42.0 e recomendações C144 das
demais lacunas de evidência. Sem afirmar conformidade integral ou fechar fase 8.
