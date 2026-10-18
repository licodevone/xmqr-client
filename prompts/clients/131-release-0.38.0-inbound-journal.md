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

## v1.1 — execução isolada e migração verificável

Entrada confirmada f3c71620d5a3d715fbfb58b3e8f09d69a026f854; clone novo
`.artifacts/p227-client-release038`, branch `release/0.38.0`, origin local
somente leitura. Push explícito para GitHub, sem mudar origin ou C128.
Consultas remotas não encontraram v0.38.0 nem imagem/tag0.38.0 em2026-10-18.

CI preserva jobs existentes e acrescenta release/** e C129:181Rust atuais,
21TCP,6compat, contratos classificadores e migração. Para migração real no
CI, checkout histórico fixo5ba9cf67a36b3c196bd1b60b35b74819d4768c47
(ancestral commitado do draftv1), build somente mqtt-client em target próprio;
não inventar documento antigo editando o campo version. Local usa ELF109ec973
congelado, com SHA no relatório. Nenhuma dependência nova.

Versionar somente pacote/client038, Docker default038 e docs de distribuição;
bridge fonte permanece no repo e nos gates herdados, mas sem binário/unit
bridge no pacote/imagem. Coletar notices pela fonte/lock exatos. Compilar
release com um job no cache privado; Debian nativoUbuntu26 existente e
container Ubuntu26 descartável para instalação/upgrade036→038/rollback/purge.

Rollback do pacote NÃO é compatibilidade do journal inbound:036 publicado
não conhece C129 e NÃO deve abrir diretório com mensagens pendentes novas.
Seed real de journalv2 com o038 instalado, guardar hashes privados, voltar036
sem iniciar assinatura sobre esse diretório; provar bytes/permissões preservados
e restaurar038 antes de retomar. Não apagar journal para tornar downgrade verde.
Configuração/credencial sintética preservadas em todo ciclo. Sem systemd host.

Fixtures Docker usarão imagem por ID/digest imutável, volumes/rede privados
sem portas host e usuário10001:10001. TCP/mTLS e WSS dos testes existentes;
C129 TCP por wrapper Docker em rede host do Linux VM somente loopback privado
ou runner descartável compartilhando namespace, com paths privados/mode0700.
Registros devem distinguir pub/sub por digest de fixtures sintéticas do journal,
sem inferir brokerE/v16/admin final ou processamento de negócio.

Antes de correção após qualquer FAIL de gate, acrescentar revisão e preservar
relatório/estágio/exitcode. CI/pacote/imagem/local PASS antes de tag/prerelease;
assets remotos baixados para pasta nova e comparados por tamanho/SHA256.

## v1.2 — revisão de proveniência antes da migração no CI

O coordenador detectou que5ba9cf6 existe apenas na história do clone de
aceite: os commits chegaram ao GitHub por cherry-pick. A consulta local
cat-file não prova alcance pelo GitHub; a alegação de ancestral emv1.1 estava
incorreta. A primeira fonte512f85e foi enviada para CI antes dessa revisão;
preservar o resultado se falhar nesse estágio, sem atribuir a runtime.

Usar o equivalente público235f20d097b674fadcbdf85d8c8f1c581ff1eba9,
cujo diff src/vendor/tests com5ba9cf6 é vazio. CI checkout full history contém
esse ancestral de mainf3c7162. Nenhum oráculo de migração muda; somente a
referência histórica. Acrescentar -j1 ao build Docker para honrar orçamento.
