# Histórico de versões

## 0.11.0 — TLS com senha sem certificado de cliente, experimental

- C48/B10: `--tls-password` e `tls_password = true` no perfil TOML v1.
  CA/SAN/validade continuam obrigatórios; cert/key e identidade implícita são
  recusados nesse modo. Senha permanece privada; mTLS continua padrão.
- Integração com broker P97: QoS0/1/2, negativas de CA/SAN/senha/ACL.

## 0.10.0 — perfis de conexão, experimental

- C47: `--profile PATH` carrega TOML v1 para parâmetros de conexão mTLS.
  Flags explícitas prevalecem; certificados, chave e senha relativos ao perfil
  resolvem a partir de sua pasta. Sem perfil, a CLI mantém o contrato anterior.
- Perfis limitados a 64 KiB e schema fechado; não aceitam segredo inline,
  payload nem modo de laboratório. Integração real com dois endpoints do broker,
  QoS0/1/2 e negativos de CA/credenciais.
- Rotação de certificados e opções de TLS sem certificado seguem pendentes.

## 0.9.0 — robustez de sessão e distribuição experimental

- C45: `--total-timeout-seconds` 1..86400 opcional, monotônico, sem reset por tráfego/retry.
- Jitter 80..100% no backoff exponencial, orçamento 0..10 total sem reset.
- CONNACK de reconexão limitado pelo protocolo; SUBACK/idle com deadline absoluto.
  PINGRESP/ACKs não renovam os 90s de ausência de PUBLISH.
- SIGINT/SIGTERM prioritários; não reconectar/republicar no cancelamento ou prazo total.
- Duplicata QoS1 continua legítima; recepção QoS2 interrompida falha explicitamente.
- Aceite técnico coordenado em 2026-10-13: 63 testes do client, interop 87/87,
  ensaio do broker 6/6 (três repetições TCP/mTLS >=600s), QA de cinco payloads
  e 14 negativos, ciclo Ubuntu26 com execução real dos ELFs instalados.
- C46: fonte e tag v0.9.0 em `e688c75a3779803e8f1868792dcc5e7a36f2dd6e`;
  [prerelease publicada](https://github.com/licodevone/xmqr-client/releases/tag/v0.9.0)
  em 2026-10-13, com 18 anexos conferidos por nome/tamanho/SHA.
- `.deb` Ubuntu26 amd64 e imagem Bookworm Linux/amd64 distribuídos. O tar.gz
  é export Docker para `docker load`, não pacote de executáveis nativos.
- Docker Hub `lepinheiro100/xmqr-client:0.9.0` verificado remotamente por digest,
  pull/run, versão real e revision; downloads de `.deb`/manifest e sidecars
  coincidem com os locais. Não altera latest nem tags/assets anteriores.
- MQTT 3.1.1 e MIT preservados; sem novas dependências ou relaxamento de
  TLS/SAN/credenciais/limites. Não declara produção, SLA ou conformidade integral.
- Evidências e limitações: [registro C46](prompts/registros/C46-release-fase3.md).

## 0.8.1 — distribuição Docker Hub

- Corrige label Docker que ainda identificava 0.7.1; verifica a versão real no build.
- Contexto allowlist, artefatos/checksums/proveniência de commit aceito.
- .deb Ubuntu26 amd64 e imagem Bookworm amd64 distribuídos separadamente.
- Plano compartilhado concluído; sem mudança funcional do client 0.8.0.

## 0.8.0 — filtros retained na saída

- `sub --ignore-retained` e `sub --retained-only`, exclusivos e opcionais.
- `--count` considera somente mensagens exibidas; mensagens filtradas mantêm
  os ACKs de QoS1/2. Default, assinatura, ACL e formato de saída preservados.
- Flags baseadas no RETAIN recebido, com distinção entre replay e publicação
  ao vivo documentada. Fixtures e integração real cobrem QoS0/1/2.

## 0.7.1 — distribuição Docker

- LabelsOCI/version/revision, SIGTERM e imagemLinux/amd64 exportada comchecksum.
- mTLS pub/sub, retained/restart e negativos em containers isolados aprovados.
- Pacote.debUbuntu26 porrelease, com install/upgrade/rollback testado.
- Sem alterações funcionais de protocolo, TLS ou quotas.


## 0.7.0 — C39 aceite Ubuntu da fase1

- SUBSCRIBE completo65536/65537 e payload4096/4097 com negativos UTF-8/QoS0/1/2.
- Recepção com corpo até65531 e reserva de header5 para manter frame até64KiB. Pode recusar frame canônico65536 por1byte; não afeta tráfego suportado pelo broker.
- Diagnóstico de SUBACK orienta ACL/limites sem atribuir razão inexistente em MQTT3.1.1.
- Harness Unix real isolado para quotas/C37 aprovado no registroC39.

-52 testes Ubuntu, fmt/Clippy/build e integração com broker0.11.0 aprovados.
- Deb Ubuntu26 amd64; install/upgrade/rollback/remove/purge aprovados em container.

## 0.6.0 — C37 timeout e keepalive

- `--keepalive-seconds` 1..65535 (padrão 30), preservado na reconexão.
- `--timeout-seconds` 1..300 (padrão 8) para protocolo; idle/input/cancelamento preservados.
- Fixtures locais cobrem CONNECT, CONNACK silencioso, SUBACK, PUBACK/PUBCOMP e QoS2 recebido.
- Gates Windows aprovados; Unix, MSRV 1.88 exato e interoperabilidade externa pendentes.
- Marco de código portátil; sem novo pacote Ubuntu. Registro: `prompts/registros/C37-timeout-keepalive.md`.

## 0.5.0 — systemd e Ubuntu 26.04

- Tratamento SIGTERM/SIGINT e integração com systemctl.
- Pacote .deb amd64 para Ubuntu26.04, sem ativação automática.
- Configuração e dados preservados; guia de instalação, upgrade e remoção.
- Release experimental com pacote e SHA-256; sem repositório APT próprio.
- DISCONNECT limitado no encerramento; units sub e pub oneshot opcionais.
- CI Linux para fmt, testes, Clippy, build e inventário.
- Evidência: prompts/registros/C31-systemd-ubuntu.md.

## Histórico dos marcos anteriores

# Changelog

## 0.4.0 — tag publicada v0.4.0

- C29: `sub` aceita até 256 filtros por repetição de `--topic`, numa assinatura.
- Pacote SUBSCRIBE limitado a 64 KiB; SUBACK exige uma concessão por filtro.
- `pub` continua aceitando um único tópico; mesmo QoS se aplica ao lote.
- Integração C29 exercita filtros disjuntos/sobrepostos, retained e `$` com XMQR.
- Evidência: `prompts/registros/C29-multiplos-topicos.md`.
- Gates Windows e Ubuntu-26.04/WSL aprovados; Rust 1.88 exato não validado.
- C30: imagem Docker local multi-stage Debian slim, entrypoint `mqtt-client`;
  `xmqr-client:0.4.0` construída sem publicação em registry.
- Evidência Docker: `prompts/registros/C30-docker.md`.

## 0.3.0 — tag publicada

- C28: --message-file e --stdin preservam payload binário até4096 bytes.
- --line-mode publica serialmente por LF/CRLF, preserva linhas vazias e EOF sem LF.
- Backpressure, espera de entrada limitada8s e cancelamento sem travar runtime.
- Conclusões text/JSONL por linha; falha parcial não reenvia mensagens.
- --version e manifest/lock alinhados à versão própria0.3.0; dependências sem upgrade.
- Próxima tag sugerida: **v0.3.0**. Não criada/enviada por esta tarefa.
- Evidência: prompts/registros/C28-payload-binario.md; MSRV1.88 e interoperabilidade
  externa completa não comprovados.

## v0.2.0 — tag existente, commit6d25b33

- Inclui C26 (reconexão opcional limitada e JSONL) e C27 (contrato/ensaio bootstrap).
- O manifest nesse tag ainda indicava0.7.0 herdado da origem. C28 corrige adiante,
  sem reescrever tags/histórico. Existência da tag remota não comprova GitHub Release.

## Extração inicial

- Cliente Rust2024/MIT independente do broker; binário mqtt-client preservado.
- Versão0.7.0 herdada, descrita historicamente em ORIGIN.json.
