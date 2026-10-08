# Cobertura atual dos prompts do cliente

Snapshot C27/C26, 2026-10-08. Código presente não equivale a validação externa.
Pacote/binário independentes, Rust2024/MSRV1.88, MIT, manifest0.7.0 local sem release.
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
| Próxima fatia payload binário | C28 PLANNED | Arquivo/stdin têm critérios, mas NÃO existem ainda no cliente atual. Sem timestamps/perfis/múltiplos tópicos. |

Gates atuais: fmt --all -- --check; test --locked; clippy --locked --all-targets
-- -D warnings; build --locked. Execução offline usa o lock/cache, sem upgrades.
MSRV1.88 declarado; validação Windows1.98.1/Linux1.99.0; toolchain1.88 não instalado, sem instalação. Nenhuma dependência do metadata declara MSRV acima1.88, mas isso não substitui compilar com1.88.
Scripts de integração do broker devem receber --client explícito e rodar somente
após coordenação em ambiente isolado; não executar estado real/durável aqui.

Registros: [C25](registros/C25-extracao.md), [C26](registros/C26-reconexao-jsonl.md), [C27](registros/C27-bootstrap.md).
