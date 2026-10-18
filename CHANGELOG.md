# Histórico de versões

## Não publicado — C116 e C117 em validação

- C117 exige `--allow-remote-wss` para URL WSS não loopback e mantém CA/SAN, certificado cliente, credenciais e subprotocolo seguros; sem fallback WS.
- O perfil TOML v1 aceita `ws_url` WSS mTLS em loopback e `--ws-url` explícito pode substituir a URL do perfil; conflito com host/port, TLS-senha e estado durável é recusado. Sem novo estado nem alteração do pacote 0.33.0.

## 0.33.0 — WSS mTLS local (experimental)

- `--ws-url wss://127.0.0.1:PORTA/mqtt` opt-in no modo mTLS seguro usa a CA/certificado/chave existentes para pub/sub MQTT 3.1.1/5. Loopback apenas, sem perfil ou estado durável sobre URL; TLS/senha e WS aberto preservados.

## 0.32.0 — WebSocket MQTT de laboratório (experimental)

- C112: `pub`/`sub` MQTT 3.1.1/5 aceitam `--ws-url` com `--open-lab` em IP loopback e caminho explícito. O handshake exige subprotocolo `mqtt` e a CLI rejeita URL externa, credenciais na URL, estado persistente e destino TCP simultâneo.
- WSS, perfis WS e durabilidade em URL WS continuam pendentes. O broker XMQR 0.39.0 oferece WS local P210; fase 7 global aberta.

## 0.31.0 — confirmação após saída da assinatura persistente (experimental)

- C110: com `--subscription-state-dir`, PUBLISH MQTT 5 QoS 1/2 recebe PUBACK/PUBREC somente após validação, filtros e escrita/flush em stdout. Falha de saída não confirma; broker pode reenviar na sessão persistente. Mensagens filtradas são confirmadas.
- O manifesto não guarda mensagens recebidas. Flush não garante gravação no consumidor; queda após flush e antes de ACK pode duplicar a saída. Sem promessa de processamento exatamente uma vez. M03/6B e demais fases globais permanecem abertas.

## 0.30.0 — outbox QoS 2 e declaração de assinatura persistente (experimental)

- C107: `--outbox-dir` aceita uma publicação MQTT 5 QoS 2 por diretório Linux/WSL. O cliente grava a transição antes do PUBREL, retoma PUBLISH/DUP ou só PUBREL conforme o estágio e exige Session Present.
- C108: `sub --subscription-state-dir` guarda a declaração de uma assinatura contínua e, após SUBACK positivo, retoma a sessão sem novo SUBSCRIBE quando Session Present=1. Falha preservando o manifesto se a sessão sumiu ou a declaração mudou.
- Mensagens recebidas não são persistidas localmente; QoS 2 confirma o broker, não o processamento por assinantes ou destinos externos. Fases 6A–6D/7–9 continuam abertas.

## 0.29.0 — sessão no DISCONNECT e outbox QoS 1 (experimental)

- C104: `--disconnect-session-expiry-seconds` envia Session Expiry Interval no DISCONNECT normal MQTT 5; opção omitida mantém os bytes anteriores.
- C105: `--outbox-dir` opt-in no Linux/WSL grava uma publicação MQTT 5 QoS 1 antes da rede, preserva resultado incerto e só retransmite DUP com o mesmo Packet Identifier quando o broker confirma Session Present. QoS 2 e assinaturas locais duráveis seguem pendentes; o journal privado contém o payload em claro.

## 0.28.0 — diagnóstico e propriedades de assinatura MQTT 5 (experimental)

- C101: a CLI informa nome e código do DISCONNECT enviado pelo broker sem ecoar Reason String não confiável. Teste TCP cobre `0x95` e texto sentinela; sem tag isolada.
- C102: `sub` MQTT 5 aceita User Property repetível em SUBSCRIBE e UNSUBSCRIBE, inclusive junto de Subscription Identifier; a segunda flag exige `--count N`. Limites e interoperabilidade XMQR/Mosquitto validados.

## 0.27.0 — Client Identifier atribuído no MQTT 5 (experimental)

- C99: `--server-assigned-client-id` permite Session Expiry finita ou infinita com Clean Start verdadeiro. O ID mostrado em stderr pode ser reutilizado manualmente com C98; a CLI não persiste estado QoS local.
- C98: `--client-id` aceita até 254 caracteres ASCII em MQTT 5 para reutilizar o ID atribuído pelo broker; MQTT 3.1.1 mantém 23. Não há retomada automática do estado QoS local.
- C97: `--server-assigned-client-id` solicita ID no broker MQTT 5, valida CONNACK obrigatório e único e exibe ID somente em stderr. O parser vendorizado rejeita duplicata da propriedade `0x12`.

## 0.26.0 — metadados e expiração do Will MQTT 5 (experimental)

- C95: `--will-message-expiry-seconds` envia Message Expiry Interval no Will MQTT 5, com validação pré-rede e sem confundir com a expiração do PUBLISH. Broker parceiro P164 inicia o prazo na publicação.
- C94: opções explícitas para Payload Format Indicator, Content Type, Response Topic, Correlation Data hexadecimal e User Property repetida no Will MQTT 5; validação antes da rede, ordem dos pares preservada e combinação com Will Delay. Sem Message Expiry do Will.

## 0.25.0 — SASL PLAIN e reautenticação explícita MQTT 5 (experimental)

- C92: `--reauth-before-io` em MQTT 5 PLAIN/TLS envia AUTH `0x19`, aguarda AUTH `0x00` com o mesmo método e só então publica ou assina. Timeout, método divergente e resposta inválida encerram sem expor senha. Patch local de `rumqttc` 0.25.1 documentado em `vendor/rumqttc-0.25.1/XMQR-PATCHES.md`; reautenticação periódica continua ausente.
- C91: `--auth-method plain` em MQTT 5 sobre TLS envia Authentication Method/Data SASL PLAIN sem duplicar Username/Password; exige confirmação `PLAIN` no CONNACK. Senha continua fora dos argumentos, por prompt ou arquivo privado. AUTH posterior depende da opção explícita C92.

O broker candidato 0.28.0/P160 é o par deste marco. Reautenticação periódica, desafio AUTH e retomada automática de fluxo MQTT 5 após restart continuam ausentes.

## 0.24.0 — shared subscriptions MQTT 5 (experimental)

- C89: `sub --protocol-version 5 --topic '$share/grupo/filtro'` valida grupo/filtro, recusa No Local e falha antes do SUBSCRIBE quando o CONNACK anuncia shared subscriptions indisponíveis. Integração externa com Mosquitto 2.0.22 e broker XMQR após P153/P154; broker 0.25.0 publicado ainda não implementa o roteamento compartilhado.

## 0.23.0 — Subscription Identifier MQTT 5 (experimental)

- C87: `sub --protocol-version 5 --subscription-identifier N` envia propriedade MQTT 5 0x0B para todos os filtros do SUBSCRIBE, após validar 1..268435455 e o anúncio no CONNACK. JSONL preserva a lista de IDs do PUBLISH, inclusive duplicatas; ID zero recebido é recusado. Mosquitto 2.0.22 foi validado para filtros simples e sobrepostos. Broker XMQR implementa P150 desde 0.25.0.

## 0.22.0 — opções de assinatura e Topic Alias (experimental)

- C83: `pub` MQTT 5 pode usar `--publish-topic-alias 1..16` com repetição sequencial; exige negociação no CONNACK, estabelece tópico completo e envia tópicos vazios na mesma conexão. Ausência ou limite insuficiente falha antes da primeira publicação.
- C84: `sub --protocol-version 5 --retain-handling 0|1|2` envia a opção Retain Handling por filtro; os filtros locais de saída continuam independentes.
- C85: `sub --protocol-version 5 --no-local --retain-as-published` envia os bits de opção por filtro; a validação recusa publisher e MQTT 3.1.1 antes da rede. O broker XMQR 0.24.0 implementa P148.

## 0.21.0 — metadados no assinante e repetição MQTT 5, experimental

- C80: `sub --protocol-version 5 --output jsonl` exibe propriedades PUBLISH recebidas, inclusive Correlation Data binária em hexadecimal e User Property como pares ordenados; formatos text/hex/raw e MQTT 3.1.1 mantidos.
- C81: `pub --protocol-version 5 --repeat` publica uma fonte única em série, aguardando confirmação antes da próxima e reaplicando propriedades. Erro ou timeout interrompe o lote sem replay automático. Fluxos QoS locais não são retomados.

## 0.20.0 — metadados de publicação MQTT 5, experimental

- C77: `pub --protocol-version 5 --payload-format 0|1 --content-type TEXTO` envia as propriedades MQTT 5 junto com Message Expiry opcional. `--payload-format 1` exige payload UTF-8 antes de abrir conexão. A saída de `sub` ainda não exibe essas propriedades.
- C78: `pub --protocol-version 5` aceita Response Topic, Correlation Data hexadecimal e User Property repetível; valida antes da rede, preserva ordem e nomes repetidos. O assinante ainda não exibe metadados.

## 0.19.0 — recepção de Topic Alias MQTT 5, experimental

- C75: `sub --protocol-version 5 --topic-alias-maximum 0..16` anuncia o limite de Topic Alias recebido. O client reconstrói o nome do tópico para filtros e saída quando o broker envia a forma abreviada; um processo novo não reutiliza associações antigas. A publicação do client ainda não comprime tópicos. O Mosquitto 2.0.22 falha ao receber a forma abreviada válida em fixture independente; não se afirma interoperabilidade nessa direção.

## 0.18.0 — limites de recebimento MQTT 5, experimental

- C72: `sub --protocol-version 5 --receive-maximum` anuncia a quota de PUBLISH QoS1/2 em voo.
- C73: `sub --protocol-version 5 --maximum-packet-size` anuncia o maior pacote MQTT aceito, até 64 KiB.
- O broker parceiro 0.20.0 aplica ambos; fases 6A–6D seguem abertas.

## 0.17.0 — validade de publicação e Will MQTT 5, experimental

- C70: Will MQTT 5 com `--will-delay-seconds` opcional, validado e enviado no CONNECT.
- C69: `--message-expiry-seconds` em `pub --protocol-version 5` envia Message Expiry Interval.
- Novo processo do client ainda não retoma automaticamente seus fluxos QoS pendentes; fases 6A–6D seguem abertas.

## 0.16.0 — sessões MQTT 5, experimental

- C67: `--session-expiry-seconds 0..4294967294|never`, validado e enviado no CONNECT MQTT 5.
- C66: `--clean-start true|false` independente da expiração, rejeição de opções de sessão 3.1.1 em MQTT 5 e CONNACK incoerente.

## 0.15.0 — MQTT 5 ampliado, experimental

- C64: flags MQTT 5 para Request Problem/Response Information e pares repetíveis de User Property no CONNECT; validação local de valores e limite 4096 bytes, sem alterar MQTT 3.1.1.
- C63: `pub`/`sub --protocol-version 5 --qos 2` completam o ciclo QoS2; publicação aguarda PUBCOMP e mostra motivo/código de PUBREC ou PUBCOMP recusado.
- C62: após `sub --protocol-version 5 --count`, cancela cada filtro, valida UNSUBACK e só então envia DISCONNECT; recusa ou timeout causa erro.
- C61: CONNACK MQTT 5 recusado mostra motivo e código hexadecimal, incluindo `BadUserNamePassword (0x86)`; integração real confirma SUBACK ACL `NotAuthorized (0x87)` do broker P120.
- C60: erros de PUBACK/SUBACK MQTT 5 mostram motivo e código hexadecimal, incluindo `NotAuthorized (0x87)`; teste cruzado com broker P119, sem reenvio automático.

## 0.14.0 — MQTT 5 inicial, experimental

- C58: `sub --protocol-version 5` com filtros QoS0/1, SUBACK validado,
  mensagens retidas/vivas e `--count`; sem propriedades avançadas ou QoS2.
- C57: `--protocol-version 5` para publicação QoS0/1 com Clean Start, sem
  Will, lote ou reconexão; aceita a mesma escolha em perfil/ambiente. O padrão
  MQTT3.1.1 e os seus recursos permanecem. A fase 6A integral continua aberta.

## 0.13.0 — opções pub/sub da fase 5, experimental

- C52/C03: `--exclude-topic` filtra a saída por tópico, inclusive com `+/#`;
  `--count` considera apenas mensagens exibidas, mantendo ACKs QoS1/2.
- C53/C04: `--repeat` com intervalo finito monotônico para uma fonte de
  publicação; aguarda o protocolo antes da próxima e não repete resultado
  desconhecido.
- C54/C06: saída `hex` reversível e `raw` binária para uma mensagem, com
  status fora do stdout; text/jsonl preservados.
- C55/C08: Will binário por arquivo regular até 4096 bytes, validado antes
  da rede, sem expor caminho ou conteúdo nos erros.

## 0.12.0 — perfis reutilizáveis completos, experimental

- C50/C05: adiciona `clean_session`, `reconnect_attempts` e
  `total_timeout_seconds` ao TOML v1, com tipos estritos e precedência
  flag CLI > ambiente explícito > perfil > default.
- Variáveis `MQTT_CLIENT_*` cobrem conexão, identidade, prazo e reconexão;
  `MQTT_CA_CERT`/`MQTT_DEVICE_CERT`/`MQTT_DEVICE_KEY` também vencem o perfil.
  Caminhos de ambiente e flags seguem relativos ao cwd; senha só por prompt
  oculto ou arquivo privado no Linux.
- Integração real mTLS e TLS-senha com broker P101/P97, negativos de valor e
  schema, sem mudança no wire MQTT.

## 0.11.1 — correção de distribuição do C48

- C49: alinha `ARG VERSION` padrão do Dockerfile ao Cargo. Nenhuma alteração
  de TLS/MQTT. A tag0.11.0 de código não teve release por falha no contrato de
  distribuição; a publicação usa este patch com novo commit e tag.

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
