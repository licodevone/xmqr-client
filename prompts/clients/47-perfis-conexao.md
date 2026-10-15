# C47 rev1 — perfis de conexão

2026-10-15. Entrada6ae0189/client0.9.0, árvore limpa. Objetivo: --profile PATH
carrega TOML version=1 e [connection] para host/port/client-id/username e
ca/cert/key/password-file, qos/keepalive-seconds/timeout-seconds e output.
Campos TOML usam snake_case. Somente mTLS padrão; sem senha inline, payload,
Will, modo laboratório ou shell. Paths relativos à pasta do perfil.

Escopo: src/profile.rs, cli.rs, testes CLI/wire/integração com P96, README,
matriz/checklist, licenças/manifests0.10.0 e registroC47. Dependências serde/toml
com versões compatíveis já usadas pelo ecossistema. Perfil<=64KiB, arquivo regular,
schema fechado; erros não incluem conteúdo. Flags explícitas prevalecem sobre
perfil; variáveis existentes continuam fallback abaixo do perfil. Duplicações
explícitas seguem recusadas; --profile único. Sem perfil, contrato atual idêntico.

Aceite: negativos de schema/secret/versão/tamanho/duplicação; precedência CLI e
paths; teste real com broker separado e mTLS/CA/SAN,1perfil por endpointP96.
Gates fmt/test/clippy/build/inventário Ubuntu26; credenciais e recursos temporários.
Migração: opt-in, sem mudança de estado/wire. C04 parcial até rotação/TLSB10;
não declarar fase4 concluída. Tag0.10.0 após aceite, release com pacote testado.

## rev2 — lockfile

O primeiro `cargo test --locked --no-run` recusou o `Cargo.lock`: as novas
dependências declaradas ainda não constam do lockfile. Atualizar o lockfile
somente para resolver `serde` e `toml`, mantendo o pin das demais dependências;
repetir o gate com `--locked` e revisar o diff antes de prosseguir.

## rev3 — revisão da resolução e warning

`cargo generate-lockfile --offline` compilou, porém atualizou versões antigas
além das novas dependências e emitiu `unused_imports` no teste do perfil.
Restaurar apenas o lockfile gerado nesta tentativa e deixar Cargo acrescentar
o mínimo ao lock anterior; remover o import supérfluo. Repetir gates travados.

## rev4 — formatação

A resolução incremental acrescentou apenas 13 pacotes e preservou os anteriores.
`cargo fmt --check` encontrou estilo pendente em `main.rs` e `profile.rs`.
Aplicar `cargo fmt`, repetir testes e lint; manter a inspeção do parser como
parte do aceite funcional.

## rev5 — função de expansão

Os 65 testes passaram. Clippy recusou 105 linhas em `profile::expand` (limite
100). Extrair a conversão de cada campo TOML em helper, sem alterar precedência
nem mensagens sanitizadas; repetir Clippy/build e integração real.

## rev6 — ambiente do inventário

A primeira chamada do inventário não tinha Cargo no PATH do subprocesso Python.
Repetir com `/home/licod26/.cargo/bin` no PATH; caso as dependências novas
apareçam, atualizar o inventário conscientemente e repetir `--check`.

## rev7 — licenças das dependências novas

Com PATH correto, broker/admin mantiveram inventário; client mudou pelo
acréscimo de serde/toml e transitivas. Conferir as declarações de Cargo metadata,
regenerar `docs/third-party-licenses.md` com o script do projeto e repetir o
check. Preservar os textos originais na distribuição.
