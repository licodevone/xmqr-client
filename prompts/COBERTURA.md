# Cobertura atual dos prompts do cliente

## C139–C143 — candidata 0.42.0, 2026-10-18

Bridge MQTT5/QoS2 C139 já integrado na fonte anterior. C140–C142 corrigem
cinco falhas obrigatórias P232 com prompts anteriores ao código, testes de
capacidades/sessão e receptor QoS2 público + wire. C143 registra RED/GREEN,
regressões e distribuição separadamente: [registro](registros/C143-five-audit-fixes.md).
Não apaga a auditoria histórica, não fecha as quatro recomendações recebidas
nem declara conformidade/certificação global. Checkpoints abaixo são históricos.

## C45/C46 — aceite técnico da fase 3, 2026-10-13

Client **0.9.0 aceito tecnicamente e distribuído no recorte experimental**
por C46/P92 v1.7–v1.9.
Fonte dos artefatos: `e688c75a3779803e8f1868792dcc5e7a36f2dd6e`, clone clean.
Tag v0.9.0, prerelease/18 anexos e Hub numerado verificados remotamente;
downloads de `.deb`/manifest e sidecars coincidem com os locais. Commits
documentais posteriores não mudam a fonte atribuída ao pacote.

| Contrato atual | Evidência coordenada final | Limite |
| --- | --- | --- |
| Robustez C07 | 63 testes do client, idle 90s real, SIGINT/SIGTERM, jitter/budget e prazo total monotônico opcional; [C45](registros/C45-robustez-reconexao.md) | Sem replay de publicação incerta; recepção QoS2 interrompida falha explicitamente |
| Interoperabilidade mTLS/ACL | 87/87 com XMQR/Mosquitto 2.0.22, dois brokers e sentidos, reconexão/crash/idle; [C46](registros/C46-release-fase3.md) | Cenários finitos; não conformidade integral ou SLA |
| Pressão/recuperação do broker integrado | 6/6, três repetições TCP/mTLS >=600s; 1808,946s de parede | Ensaio do broker, não benchmark do client ou estabilidade ilimitada |
| Fonte e distribuição local | Oito gates do clone clean e688; `.deb` Ubuntu26 amd64; ciclo install/upgrade/rollback/remove/purge com ELFs realmente executados | Não homologa Ubuntu24/arm64/Windows nativo/MSRV1.88 exato |
| Licenças e payloads | QA 5/5 +14 negativos; client 81 pacotes/158 textos originais preservados, MIT própria canônica | Superset do lock, não composição completa do ELF ou auditoria jurídica |
| Publicação C46/P92 | [Prerelease v0.9.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.9.0), 18 anexos conferidos; Hub 0.9.0 por digest/pull/run/revision; downloads reais conferidos | Sem latest/overwrite; tar.gz é imagem para docker load, não ELF nativo |

MQTT 3.1.1/MIT e os limites existentes permanecem. MQTT 5 é planejamento da
fase 6, não iniciado; fases 4/5 não foram implementadas por este fechamento.
Veja o [registro C46](registros/C46-release-fase3.md) para fontes, hashes,
falhas históricas preservadas e verificação da publicação.

## Snapshot histórico preservado

C43/C44 encerraram a fase2 no recorte aceito. A tabela seguinte descreve o
snapshot C29/C28/C27/C26; suas lacunas antigas não substituem o aceite atual.

Snapshot C29/C28/C27/C26, 2026-10-10. Código presente não equivale a validação externa.
Pacote/binário independentes, Rust2024/MSRV1.88, MIT, manifest0.4.0 candidato independente sem nova release.
Nenhuma dependência do crate do broker. Estado de origem em ORIGIN.json.

| Contrato | Fonte/prompt atual | Evidência e lacuna |
| --- | --- | --- |
| Crate/build independente | Cargo.toml, Cargo.lock, C25 | Gates Windows e Linux; bootstrap novo C27 compilado/testado com insumos explícitos, sem fonte broker. Resultados completos no registro C27. |
| Dependências | Manifest/lock, docs/third-party-licenses.md | rumqttc=0.25.1 (use-rustls-no-provider), rustls0.23.45 (aws_lc_rs/logging/prefer-post-quantum/std/tls12), rustls-pemfile2.2.0, rpassword7.5.4, tokio1.53.1 (io-util/macros/net/rt-multi-thread/signal/sync/time), tempfile3.27.0 dev. Lock preservado, inventário PASS, sem novas dependências. |
| pub/sub e CLI | src/cli.rs, C24/C26 | Defaults e validação testados; qos0/1/2, retained, count, ID e Will presentes. |
| Limites UTF-8 | src/cli.rs | Tópico/filtro1024 e mensagem/Will4096, usuário128, senha1024; testes de limites CLI PASS. Não confundir com limite de frame recebido64KiB. |
| TLS/SAN/mTLS | src/tls.rs, C24 | CA explícita e identidade cliente, sem insecure; leitura PEM1MiB testada. Peer TLS próprio em Linux testa mTLS válido e CA/SAN negados; não é broker TLS de produção ou interoperabilidade externa. |
| Credenciais | src/credentials.rs/main.rs | Senha oculta; Linux password-file regular/current owner/0600 e máximo1026 com CRLF. Gates Linux/0600 executados no Ubuntu26; fixture TLS usa /tmp nativo porque DrvFS recusou0600 corretamente. Credenciais não entram no JSON. |
| Laboratórios | src/cli.rs | open-lab anônimo e plain-auth-lab somente127.0.0.1/::1; fixtures somente loopback efêmero. |
| QoS e confirmações | src/session.rs, C22/C24/C26 | Fixtures pubQoS0/1/2 e subQoS1/2 PASS; ACK confirma protocolo, não negócio. Sem replay de pub interrompida. |
| Sessão e reconexão | C26/session.rs | Mesmo ID/EventLoop, orçamento total0..10, espera100ms..5s, auth/protocolo sem retry; sessão presente/ausente, EOF e limites finitos testados. Sem estado em disco. |
| Recepção QoS2 interrompida | C26rev1.2 | Limitação real rumqttc: clean perde estado. Cliente falha explicitamente nessa fase; fixture PASS. Não prometer retomada/exactly-once através dessa queda. |
| Retained/Will | C22/C24/C25 | CLI/last_will existentes e flag retained wire testada. Nove cenários Will/retained/restart/takeover com binário broker congelado e estado efêmero executados para C26 e C27; não pressupor ambiente de produção. |
| JSON Lines | src/output.rs, C26 | stdout dados, stderr operações; payload_bytes0..255 sem perda, Unicode/controles/vazio e conclusão pub testados. Texto default preservado. |
| Cancelamento/deadlines | session.rs, C26 | DISCONNECT live em até1s, descarta fila no cancelamento; cancelamento da espera testado. QoS2 recebido deadline8s; SIGINT real Linux testado live/backoff, inclusive Will suprimido no link live; Ctrl+C real Windows não injetado. |
| Interoperabilidade externa | C21/C23/C25 | Após coordenação, broker congelado P43 + CLI atuais em estado/portas próprios; JSON/reconnect/Will testados. Mosquitto e broker TLS completo ainda não validados. Bloqueio WSL inicial preservado no histórico. |
| Recriação pelo contrato | C27 | Implementação nova validada em pasta inicialmente vazia, dados manifest/lock/licença/testes declarados. Não é reconstrução cega ou igualdade de bits; insumos/cache/plataforma importam. |
| Payload binário/por linha | C28rev2.0 | Arquivo/stdin bruto ou LF/CRLF; limite4096, entrada antes da rede, serial ACK, timeout8s, stdin/SIGINT e falha parcial sem replay. Wire Windows/Linux e integração isolada PASS. Sem timestamps/perfis/múltiplos tópicos. |
| Múltiplos tópicos | C29 | `sub` envia um lote de 1..256 filtros; `pub` mantém tópico único. SUBACK valida quantidade e QoS. Windows/Linux e broker XMQR isolado cobrem filtros disjuntos/sobrepostos, retained e wildcard `$`; sem QoS individual ou divisão de pacote. |

Gates atuais: fmt --all -- --check; test --locked; clippy --locked --all-targets
-- -D warnings; build --locked. Execução offline usa o lock/cache, sem upgrades.
MSRV1.88 declarado; validação Windows1.98.1/Linux1.99.0; toolchain1.88 não instalado, sem instalação. Nenhuma dependência do metadata declara MSRV acima1.88, mas isso não substitui compilar com1.88.
Scripts de integração do broker devem receber --client explícito e rodar somente
após coordenação em ambiente isolado; não executar estado real/durável aqui.

Registros: [C25](registros/C25-extracao.md), [C26](registros/C26-reconexao-jsonl.md), [C27](registros/C27-bootstrap.md), [C29](registros/C29-multiplos-topicos.md).

Tag publicada v0.3.0; candidato0.4.0 / próxima tag sugerida v0.4.0. v0.2.0 já contém C26/C27.
Registros: [C28](registros/C28-payload-binario.md), [C29](registros/C29-multiplos-topicos.md). Bootstrap C27 congelado
no baseline0.2; não afirmar reconstrução independente0.3 pelo ensaio anterior.

C38 amplia negativos de payload4096/4097, fronteira do SUBSCRIBE completo65536/65537, diagnóstico de recusa sem atribuir causa específica e harness Unix real para limites/C37. Ver [registroC38](registros/C38-limites-fase1.md); integração Unix pendente não é PASS.
