# C143 v1.2 — aceite integrado e distribuição 0.42.0

Revisão após primeira fixture ampliada: 35 PASS/5 FAIL de preparação do
harness, não cinco falhas novas do protocolo. Repeat exige intervalo explícito;
diretórios duráveis precisam existir com modo 0700; manifesto de assinatura
não admite --count. Corrigir entradas do harness e preservar relatório FAIL.
Os probes originais P232 já obtiveram as cinco correções PASS (8 PASS/4 FAIL,
somente recomendações remanescentes) no candidato; não apagar o histórico.

## Distribuição planejada após aceite

O pacote inclui C139 integrado, por isso client 0.42.0/bridge 0.4.0, não uma
patch que omita a funcionalidade nova. Atualizar Cargo/lock, Dockerfile, saída
de versão da bridge, CI e release_assets. Baseline de upgrade/rollback passa
a ser a release 0.41.0/bridge 0.3.0, .deb SHA-256
`ca68df0788d62ec2d63701eea07e63a168f4cdc26b0bbd10142dc163f686e6a7`.
CI deve testar também as 40 regressões nos ELFs do pacote e arquivar relatórios.
Somente após CI exata integral PASS: baixar .deb da CI, executar pacote e
imagem numerada/revisionada, verificar checksums, publicar tag anotada
v0.42.0, DockerHub e prerelease com seis assets pelo fluxo C138. Retain As
Published, QoS2 source journal e limites de crash C139 devem constar nas notas.

WSL retornou intermitentemente 0x8007274c ao iniciar novos comandos, inclusive
antes de executar compatibilidade. Registrar como BLOCKED de infraestrutura,
repetir sem reiniciar serviços/WSL do operador; CI limpa fornece gate externo.

Entrada/autoridade C140–C142. Consolidar apenas após GREEN individual. Rodar
fmt/test/Clippy/build em Ubuntu26/WSL, fixtures independentes de capacidades,
sessão e QoS2, os probes P232 originais preservados, regressões duráveis e
bridge com broker independente XMQR/Mosquitto quando disponível. Registrar
binários/hashes/fontes e limites; quatro recomendações P232 permanecem abertas.

Arquivos: CI, README, CHANGELOG, prompts/índice/cobertura, registro conjunto,
docs de correção. Não transformar os 252 IDs em declaração de certificação.
Commit/push aceitos autorizados pelo usuário; tag/release apenas após gates
da fonte exata e artefatos coerentes. Uma nova distribuição deve incluir C139
e as correções, versão independente 0.42.0 (minor experimental), notas de
migração, .deb Ubuntu26amd64 e Docker Linuxamd64 numerado, nunca mover latest
nem tags antigas. Reusar fluxo C138 após revisar prompt específico se houver
ajuste de release/CI; se distribuição não for concluída, deixar pendência
explícita sem tag prematura. Não instalar em host nem alterar serviços reais.
