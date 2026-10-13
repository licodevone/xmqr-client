# Prazos, reconexão e cancelamento — C45 / fase 3 C07

O prazo de protocolo não é o prazo total do comando. Estas políticas operam
independentemente e a primeira que expirar encerra a operação:

| Política | Configuração | Escopo |
| --- | --- | --- |
| Protocolo | `--timeout-seconds 1..300`, padrão 8 | Cada tentativa CONNACK e etapa SUBACK/PUBLISH/QoS2/DISCONNECT |
| Inatividade | 90s, fixo | Ausência de PUBLISH recebido depois de SUBACK; PINGRESP/ACKs não renovam |
| Entrada | 8s, fixo | Cada espera por entrada de payload; inclui a primeira antes de rede |
| Total | `--total-timeout-seconds 1..86400`, opcional | Uma sessão monotônica inteira, inclusive backoff e entrada posterior |
| Encerramento por sinal | Até 1s | DISCONNECT quando possível, sem ultrapassar o total restante |

O total começa ao entrar em `session::run`, **depois** de interpretar as opções,
ler a primeira entrada de payload e ler a senha. Inclui configuração TLS, tentativas de conexão,
espera por mensagens/protocolo, entrada posterior no modo por linha e
DISCONNECT normal. Não limita a senha interativa; entrada inicial mantém 8s.
Sem a nova opção, não existe limite total. Tráfego, retries, SUBACK e retomada
não reiniciam o relógio total. Em expiração, sai com código 1 sem nova tentativa,
sem mais 1s de tolerância e sem confirmar uma publicação incerta.
Esse encerramento fecha o link sem DISCONNECT, portanto um Will configurado
pode ser disparado pelo broker. A preparação TLS síncrona conta no orçamento,
mas leituras locais e agendamento não são preemptivos: não é garantia de tempo real ou SLA.

```sh
mqtt-client sub --open-lab --topic 'test/#' --count 5 \
  --timeout-seconds 3 --total-timeout-seconds 30 \
  --reconnect-attempts 4 --client-id session-test --clean-session false
```

O exemplo é laboratório **loopback**, não muda o perfil TLS padrão. Para o uso
seguro, substitua o modo de laboratório por usuário/CA/certificado/chave e
senha oculta ou arquivo Linux 0600, mantendo verificação SAN. Não use segredo
na linha de comando. Timeouts não desabilitam a verificação TLS.

## Tentativas e jitter

`--reconnect-attempts 0..10`, padrão 0, é o orçamento total do processo, antes e
depois do CONNACK. Sucesso não devolve orçamento. O EventLoop e ClientID
originais são preservados; rumqttc inicia conexão no próximo poll, por isso
a espera é aplicada **antes** desse poll.

A base exponencial é 100, 200, 400, 800, 1600, 3200 e 5000ms, depois 5000ms. Cada
espera recebe jitter de 80..100% da base (80..100ms na primeira, nunca mais 5s).
O PRNG não criptográfico é semeado por RandomState por processo; não é uma
garantia de distribuição uniforme nem substitui autenticação/segurança.
Fixtures usam uma semente fixa para testar limites e repetibilidade do cálculo.

Somente falhas de transporte retentáveis podem consumir orçamento. Recusas
de autenticação/ACL, erros TLS/protocolo e timeout externo não criam retries.
Prazos de SUBACK/idle incluem backoff; reconexão restabelecida recebe novo
prazo de SUBACK ou idle, mas total permanece original. CONNACK de reconexão
continua limitado pelo `--timeout-seconds`, não pela inatividade de 90s.

## Cancelamento e mensagens incertas

SIGINT/SIGTERM têm prioridade quando já pendentes na sessão, inclusive em
poll, backoff e espera de entrada posterior. Não reconectam no encerramento.
Em link ativo o cliente descarta requests pendentes e tenta DISCONNECT até 1s;
sem link apenas encerra. Fechar stdin ou cancelar não bloqueia o processo
esperando a thread de leitura nativa.
Com total configurado, a tolerância de sinal também é limitada pelo tempo
total restante; se já expirou, fecha sem novo DISCONNECT.
No Windows nativo, sinais/serviço ainda
precisam homologação própria; estes aceites são Linux/WSL.

Em pub, retry só é permitido antes de enfileirar o primeiro PUBLISH. Queda,
cancelamento ou total expirado após essa etapa não desfaz a mensagem: o
resultado pode ser desconhecido e não há reenvio do lote. QoS0 confirma envio;
PUBACK/PUBCOMP confirmam somente protocolo, não o processamento de negócio.

QoS1 permite duplicatas, também durante retransmissão após reconexão.
`--count` conta mensagens exibidas, não IDs de negócio únicos. Se necessário,
deduplique na aplicação usando seu identificador/seq persistente; não inferir
exactly-once em QoS1. Na recepção QoS2, rumqttc 0.25.1 perde estado de entrada
em `clean()` na queda: entre PUBLISH e PUBREL o cliente falha explicitamente,
sem afirmar retomada segura. Não possui persistência local de sessão.

## Evidências e limites

Prompt: [C45](../prompts/clients/45-robustez-reconexao.md). Resultados reais em
[registro C45](../prompts/registros/C45-robustez-reconexao.md). Interoperabilidade
mTLS/ACL/Mosquitto e ensaios com broker são campanhas separadas P91; fixtures
wire não substituem essas evidências. Este documento não encerra sozinho a fase 3,
não inicia perfis/listeners/MQTT5 e não é declaração de conformidade integral.

No candidato local 0.9.0, passaram 63 testes Rust em debug e release, fmt,
Clippy, build e inventário. O [ensaio real de 90s](evidence/client-c45-robustness.json)
passou com 89 PINGRESP, sem renovação do idle nem tentativa adicional; também
passaram SIGINT/SIGTERM na entrada inicial com zero conexões. Publicação e
validação da cópia exata são etapas coordenadas separadamente.

O harness Linux abaixo exercita SIGINT/SIGTERM na entrada inicial sem conectar e uma
espera real de 90s com PINGRESP contínuo; duração/trocas/hash do binário são
registrados. Não substituir esse ensaio por testes de relógio simulado.

```sh
python3 scripts/verify_client_robustness.py --client CAMINHO --report ARQUIVO.json
```

Fontes oficiais:
[rumqttc EventLoop0.25.1](https://docs.rs/rumqttc/0.25.1/rumqttc/struct.EventLoop.html),
[OASISMQTT3.1.1](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html).
