# C28 — checkpoint em execução, 2026-10-08

Prompt prévio: prompts/clients/28-payload-binario-planejado.md rev.2.0, autorizado antes do código.

## PASS observado

- Tags reais local/remoto verificadas: v0.2.0, objeto anotado 1916ea192d2bb70271c572bf2624dbe1d6e3d5cd, commit 6d25b335e28ec9df1346c5a0270ef49747cf8937. Contém C26/C27. Não há v0.3.0; manifest do tag ainda herdava 0.7.0.
- Versão candidata local Cargo.toml/Cargo.lock/--version = 0.3.0. Próxima tag sugerida v0.3.0. Nenhuma tag/commit/push/release feita nesta tarefa.
- Windows Rust1.98.1: fmt check, test --locked --offline (25 unit +13 wire), Clippy all-targets -D warnings, build e inventário MIT/dependências PASS antes da última troca equivalente de asserções.
- Linux Ubuntu26.04 Rust1.99.0: fmt check e test --locked --offline (27 unit +13 wire) PASS.
- Testes novos: fontes CLI exclusivas, framing LF/CRLF/EOF/linha vazia e limite4096, raw binário arquivo/stdin QoS0/1/2, ACK antes próxima linha, falha parcial sem reconnect, entrada inválida sem rede, --version, stdin aberto timeout8s sem hang.

## FAIL corrigido, ainda exige revalidação

- Clippy Linux1.99 exigiu duas asserções assert_eq!(bytes, []) em testes novos. Ajustadas antes deste checkpoint; reexecutar Clippy/gates após ajuste. Falhas anteriores if_not_else/match_same_arms/manual_let_else corrigidas e Windows passou.

## BLOCKED / pendente

- Checkpoint antecipado solicitado pelo pai: integração C28 ainda NÃO executada. validation/C28-integration.py preparado (broker congelado explícito, peer TLS/mTLS/password0600/stdin, SIGINT e queda entre linhas), mas não validar por mera existência.
- Revalidar gates após última troca de asserções, executar integração C28 e regressão C27/Will isoladas. Não há processo de testes ativo neste checkpoint.
- README/changelog/índice/matriz/suplemento C27 ainda pendentes de alinhamento; README antigo ainda mostra0.7.0. Não publicar/taguear estado parcial.
- Toolchain MSRV1.88 não instalado; não instalar. Mosquitto/brokerTLS completo não comprovados. C27 reconstrução congelada baseline0.2 não alterada; hashes préC28 não comprovam igualdade atual.

## Arquivos C28

Cargo.toml, Cargo.lock, src/cli.rs, src/main.rs, src/session.rs, novo src/payload.rs, tests/wire.rs, validation/C28-integration.py, prompt e este registro. Mudanças externas em prompts/README.md, registrosC26/C27 e provenance.json preservadas.

Implementação: canal1 + thread nativa para stdin/file, deadlines8s, primeiro payload antes da rede, regular file validado, serial QoS terminal, logsstderr/JSONLstdout preservados, sem replay após primeiro PUBLISH. Interrupção parcial não implica rollback ou sucesso de negócio.

## Encerramento C28 — 2026-10-08 / pronto para checkpoint de publicação

Esta seção substitui as pendências do checkpoint acima. Preservado histórico
real de falhas e correções, sem retroagir autorização. Prompt rev.2.0 precedeu
edições; histórico rev.1 continua identificado como proposta antiga.

### PASS final

1. Windows Rust/Cargo1.98.1: fmt --all -- --check, test --locked --offline
   (25 unitários +13 wire =38), clippy --locked --offline --all-targets -- -D warnings,
   build --locked --offline e scripts/license_inventory.py --check. Todos PASS
   após as duas últimas correções de asserções. Linker MSVC emite aviso informativo
   de criação .lib/.exp; comandos encerram0, Clippy sem warnings.
2. Linux WSL Ubuntu-26.04 Rust/Cargo1.99.0: mesmos gates, 27 unitários +13 wire =40,
   inventário PASS. Duas unidades adicionais são políticas de segredo Linux0600.
3. Integração C28:5 testes PASS em2.109s; execução final com
   python3 -B -W error::ResourceWarning validation/C28-integration.py.
   Broker/cliente/fixtures explícitos, loopback/estado efêmero próprios.
   - raw bytes0..255 e linhas binárias/CRLF/vazia/EOF para QoS0/1/2 no broker real;
   - SIGINT antes da rede com stdin bloqueado e entre linhas, saída0 em menos2s,
     Will não observado; worker de leitura nativa não trava encerramento;
   - queda entre linhas retorna1 sem nova conexão ou replay apesar de budget10;
   - TLS/mTLS CA/SAN válidos, segredo aleatório arquivo0600 em /tmp separado do
     payload stdin binário, CONNECT autenticado e bytes exatos, sem secret na saída;
   - FIFO/diretório/device/arquivo ausente rejeitados em até2s, saída2 e sem eco do
     caminho. Política executada em filesystem Linux nativo.
4. Regressão C27:4 testes PASS em2.463s: sessão/ID/count em restart, JSONbinário
   1024UTF8/retained/QoS0..2, cancelamento Will/backoff, CA/SAN inválidos sem retry
   ou fuga de segredo. Integração Will:9 testes PASS em6.492s. Total18 casos de
   integração (cada teste pode cobrir múltiplas combinações), sem Mosquitto.
5. --version imprime mqtt-client0.3.0 (com espaço entre nome e versão) em Windows
   e integra o teste wire. Cargo.lock alterou somente versão do pacote local;
   dependências/features/inventário preservados. README, CHANGELOG, índice,
   matriz, promptC28 e suplementoC27 alinhados. ORIGIN histórico preservado.
6. Tags reconferidas no encerramento: local/remoto v0.1.0 e v0.2.0, último
   commit6d25b335e28ec9df1346c5a0270ef49747cf8937; v0.2.0 contém C26/C27;
   v0.3.0 ausente. Nenhuma mutação Git nesta tarefa.

Comando de integração, três scripts com mesmos argumentos explícitos:
~~~text
--broker /mnt/c/Users/licod/Documents/Codex/2026-10-07/task/p43-broker-bin
--client /mnt/d/projects/my-project/xmqr-client/target/c27-linux-production/debug/mqtt-client
--fixtures /mnt/d/projects/my-project/xmqr/scripts/verify_interop.py
~~~
validation/C28-integration.py e validation/C27-integration.py recebem os três.
scripts/verify_last_will.py do broker recebe --broker e --client apenas.
TMPDIR do estado efêmero: target/c27-integration-tmp; certs/segredos/FIFO usam
/tmp nativo. Não editou, reconstruiu ou executou estado real do broker vizinho;
binário congelado aprovado anteriormente pelo pai. Nenhum processo ativo residual.

### FAIL resolvido

Clippy1.98 if_not_else/match_same_arms/manual_let_else e1.99 assert_is_empty
ajustados, todos gates passaram depois. Primeira execução C28 teve warnings de
pipes Python não fechados; cleanup explícito adicionado e5 testes repetidos com
ResourceWarning tratado como erro, sem warnings. Ensaio FIFO em DrvFS retornou
Operation not supported; fixture movida para /tmp nativo e PASS. Não enfraqueceu
validação de arquivo/segredo para contornar filesystem.

### BLOCKED / limitações remanescentes

- MSRV1.88 exato não instalado/testado; declaração e metadata de dependências
  não substituem compilação real nesse toolchain. Sem instalar software.
- Mosquitto e integração TLS completa com broker independente não executados;
  TLS/mTLS desta etapa são peers próprios, além do broker real em open-lab.
- Ctrl+C real Windows não injetado; unidades cancelamento + stdin aberto8s e
  wire Windows PASS; SIGINT de processo real comprovado somente Linux.
- Reconstrução independente0.3 não executada. C27 bootstrap permanece congelado
  no baseline incluído emv0.2.0, com hashes/manifest histórico0.7.0. Checker antigo
  não comprova equivalência atual; README do ensaio esclarece essa fronteira.
- Não foram testadas todas falhas de permissão/races de arquivo do sistema,
  ou durabilidade/sucesso de negócio. Erro parcial não faz rollback/replay;
  QoS0=envio, QoS1/2=ACK protocolo. Retain +linha vazia pode apagar retained.

### Entrega e parada

Versão candidata independente **0.3.0**; próxima tag sugerida **v0.3.0**,
não criada/enviada por esta tarefa. C28 concluído; parar antesC29 para usuário
publicar. Sem commit/push/tag/release/instalação, sem novos recursos foraC28.
Mudanças externas anteriores em registrosC26/C27/índice/provenance preservadas;
apenas README do ensaio congelado recebeu aviso, sem alterar seus fontes/hashes.
