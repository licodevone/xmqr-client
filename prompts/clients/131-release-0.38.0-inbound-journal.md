# C131 v1.0 — distribuir o journal inbound como client0.38.0

Preparado2026-10-18 antes de versionar/publicar. Entrada main3c7913a,
runtimeC129 corrigido integrado c8f9b28,181Rust/21TCP/6compat/migraçãov1real
e gates documentadosPASS. Marco publicado anterior0.36.0. C128/bridge tem
versão0.37.0 e alterações próprias ainda não commitadas no original: preservar
integralmente. Escolher0.38.0, se livre nos remotos, para não apropriar0.37.0.

## Isolamento obrigatório

Criar clone próprio limpo da fonte commitada, branchrelease/0.38.0. Não
alterar/reverter/commitar Cargo/CI/Docker/README/bridge dirty do original.
Este marco publica mqtt-client/journal, não empacotamento novo da bridgeC128.
Manter GitHubmain com as correções aceitas; preparar versão e artefatos na
branchrelease isolada. Não fazer merge/pull no original sobreC128dirty.

## Escopo

Conferir versão/tag livre; Cargo.toml/lock, CHANGELOG, VERSIONING, README,
nota docs/releases/0.38.0.md e pipeline no clone. PermitirCI emrelease/**
sem removermain/PR; adicionar gateC129 completo e migraçãoreal usandoELFantigo
frozen/debquando disponível, não instalar ferramentas no host. Manifests e
labels devem refletir0.38.0 e oSHAexato. MIT própria, Apache/notice do fork
e demais licenças originais preservados. Gerar inventário/notices reais.

## Aceite e publicação autorizados

fmt/test/Clippy/buildrelease/licenças na fonte exata;21TCP/6compat/migração
echeckWindows proporcional. CIubuntu e pacote da branch devemPASS antes
de tag/release. .debUbuntu26amd64, inspeção versão/arquivos/notices e ciclo
upgrade036→038/rollback em container descartável; journal2 exige binário
compatível, não apagar dados para tornar downgrade verde.

Imagemlepinheiro100/xmqr-client:0.38.0, não-root, labels/SHA e pub/sub
TCP/WSS e journal proporcional por digest. Nunca moverlatest/tags existentes.
Só depois criar taganotada imutável/prerelease com .deb/manifests/SHA/notices,
verificar tamanhos/hashes remotos e registrar links/digests/limites reais.

Releases permanecem experimentais; fase6 global/P227E/admin16 seguem em
validação. Não anunciar produção, brokerMQTT5integral ou suporteWindows aos
journals. Se qualquer gate falhar, revisar este prompt antes de corrigir.
Agente pode executar esta distribuição explicitamente autorizada, mas
nenhuma mutação em serviços/firewall/firmware/segredos ou estado do operador.
