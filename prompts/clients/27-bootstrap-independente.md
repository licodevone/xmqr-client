# C27 — Bootstrap independente do xmqr-client

ID C27 / revisão1.0 / 2026-10-08. Status: execução autorizada, primeiro checkpoint
de qualidade antes de continuar melhorias inspiradas no Mosquitto.
O prompt é persistido ANTES de gerar qualquer fonte de reconstrução.

## Estado, autorização e escopo

HEAD26c1019, origin licodevone/xmqr-client; manifest0.7.0 local herdado,
publish=false; taglocalv0.1.0/f8999e6 preservada. Working tree contém C26:
reconexão/JSONL, README, avisos históricos, índice/matriz/testes/registro.
Não apagar esses diffs nem alterar o cliente de produção nesta etapa.
Lidos AGENTS/README/ORIGIN/skill xmqr-client-development e C19–C26.

Objetivo: transformar os requisitos reais até C26 num contrato independente
de fontes do broker e avaliar uma implementação nova em diretório vazio.
Arquivos: este prompt, validation/c27-bootstrap (crate de evidência, não nova
versão de produção), prompts/README.md, prompts/COBERTURA.md, registro C27;
planejar depois C28 payload arquivo/stdin, sem implementar C28 neste checkpoint.
Sem instalação/commit/push/tag/release, sem editar/executar broker/estado durável.

Método: declarar todos os insumos reutilizados. Permitidos: Cargo.toml/Cargo.lock
para dependências e licença MIT literal, e testes wire C26 como oráculo de contrato
(reutilização explícita de testes, não fontes de implementação). Escrever fontes
novas com arquitetura/nomenclatura própria a partir deste contrato. Não copiar
src/ nem fontes do broker, nem substituir o diretório por checkout/export pronto.
Guardar hashes de entradas e fontes atuais para comprovar produção preservada.
Mesmo autor já conhece código anterior: ensaio orientado ao contrato, não prova
cega por outro autor, reprodutibilidade geral ou equivalência bit a bit.

## Manifest e dependências

Criar crate autônomo (fora de workspace do broker), binário mqtt-client;
edição2024/MSRV1.88, MIT, publish=false, versão local0.7.0 somente compatibilidade.
Nenhuma dependência do broker, nenhum codec próprio. O lock é um insumo declarado
do conjunto de prompts: preservar como artefato, executar --locked --offline;
se cache faltar, registrar BLOCKED em vez de instalar ou mudar versões.
Sem o lock há resolução de transitivas diferente possível; prompt sozinho não
fixa bits de dependências. Manter copyright Luis E. S. Pinheiro no LICENSE.

Manifest aprovado como dado de configuração (não copiar implementações):

~~~toml
[package]
name = "xmqr-client"
version = "0.7.0"
edition = "2024"
rust-version = "1.88"
publish = false
license = "MIT"
description = "Cliente CLI MQTT 3.1.1 independente, originado do XMQR"
readme = "README.md"

[[bin]]
name = "mqtt-client"
path = "src/main.rs"

[dependencies]
rpassword = "7.5.4"
rustls = { version = "0.23.45", default-features = false, features = ["aws_lc_rs", "logging", "prefer-post-quantum", "std", "tls12"] }
rustls-pemfile = "2.2.0"
rumqttc = { version = "=0.25.1", default-features = false, features = ["use-rustls-no-provider"] }
tokio = { version = "1.53.1", features = ["io-util", "macros", "net", "rt-multi-thread", "signal", "sync", "time"] }

[dev-dependencies]
tempfile = "3.27.0"

[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
all = "deny"
pedantic = "warn"

~~~

## CLI completa até C26

Invocação: mqtt-client pub|sub [flags]. Somente --help ou -h sozinho sai0
sem pedir senha/rede. Flags desconhecidas, repetidas ou sem valor falham exit2;
modo laboratório é flag sem valor; todos demais abaixo exigem valor.
Não adicionar --version/--insecure/--password/perfis/novos recursos.

| Flag | Padrão, domínio e contrato |
| --- | --- |
| --topic | Obrigatório: topic em pub, filter em sub; UTF-8 1..1024bytes, sem NUL; usar valid_topic/valid_filter rumqttc. Pub sem wildcards; sub pode +/# válidos. Sem normalização. |
| --message | Obrigatório apenas pub: texto UTF-8 0..4096bytes; rejeitar em sub. Não tratar como arquivo/stdin no C27. |
| --count | Apenas sub, opcional u64 positivo; ausência recebe até Ctrl+C/timeout/erro. Limite total atravessa reconnect, inclusive mensagem antes SUBACK. |
| --host | 127.0.0.1; rejeitar vazio/whitespace. Secure aceita hostname/IP; labs somente strings127.0.0.1 ou::1. |
| --port | u16 1..65535; secure8883, labs1883. |
| --ca/--cert/--key | Secure obrigatórios; flag precede MQTT_CA_CERT/MQTT_DEVICE_CERT/MQTT_DEVICE_KEY individualmente. Não aceitar flags TLS em labs. |
| --username | Obrigatório em secure/plain-auth-lab, UTF-8 1..128bytes sem controle; proibido no open-lab. |
| --password-file | Secure/plain-auth-lab somente; Linux regular/current effective owner/modo exato0600; demais plataformas falham em vez de ler. Ausência exige prompt oculto. |
| --client-id | mqtt-{pub ou sub}-{PID}; 1..23bytes ASCII alnum,-,_; nunca regenerar no reconnect. Explicitar ID para estabilidade entre processos. |
| --qos | 0 default, 0/1/2 apenas. |
| --retain | Somente pub, false default; somente true/false. |
| --clean-session | true default; somente true/false. false solicita sessão do broker, não armazenamento local. |
| --will-topic/--will-message | Ou ambos ausentes, ou ambos presentes; tópico1024bytes/publicável e mensagem0..4096bytes. |
| --will-qos/--will-retain | Só com Will completo; defaults0/false, domíniosQoS/booleano. |
| --open-lab | Transporte TCP anônimo, sem username/password-file/flags TLS; loopback exclusivamente. |
| --plain-auth-lab | TCP loopback autenticado, exige username/senha, sem flags TLS; não combinar labs. |
| --output | text default ou jsonl; case-sensitive, sem outros formatos. |
| --reconnect-attempts | u8 0..10, default0; orçamento total do processo, sem reset após conexão. |

Diagnósticos devem explicar flag/categoria sem imprimir seu valor secreto.
Senha não pode ser aceita por argv; username/paths não entram no JSON.
Erro CLI/credencial exit2; falha de rede/protocolo exit1; sucesso/cancelamento exit0.
Não exigir equivalência literal de help/erros de validação nesta reconstrução;
preservar status, flags e semântica. Frases críticas abaixo são contrato C26.

## Segurança e limites

Senha1..1024bytes UTF-8, sem chars controle; prompt rpassword antes da rede.
No Linux abrir file, inspecionar metadata do handle (regular, uid efetivo lido
de /proc/self/status, permission bits &07777==0600), ler no máximo1027bytes,
rejeitar acima1026, remover uma LF final e CR opcional, validar senha sem log.
No Windows password-file falha com categoria segura. Não gravar fixtures de segredo.

PEM: read bounded até1MiB+1 e falhar acima1MiB. CA explícita, pelo menos um cert,
cadeia cliente não vazia e private_key PEM; rustls RootCertStore das CAs fornecidas,
aws_lc_rs provider, TLS1.3/TLS1.2, client_auth_cert. Transporte rumqttc TLS normal
verifica cadeia/SAN contra host sem custom verifier/insecure. Labs usam TCP.
Não imprimir key/password/NotConnAck(Packet) em logs; resposta inesperada pode
conter payload hostil. Quotas de CLI1024/4096 não equivalem ao frame recebido64KiB.
Sessão usa keepalive30s, packet size64KiB, canal de requests8; não importar broker.

## Estados MQTT e recuperação

1. Construir MqttOptions com ID/host/port, clean_session, Will, limites e credenciais
   quando autenticado; AsyncClient e EventLoop únicos durante processo.
2. CONNECT precisa de CONNACK sucesso antes de pub/sub; espera inicial8s por poll.
   Rumqttc poll conecta/reconecta por si; não abrir conexão paralela/segundo loop.
3. Pub enfileira PUBLISH exatamente uma vez. QoS0 completa em Outgoing::Publish(0),
   QoS1 em PUBACK, QoS2 em PUBCOMP com PUBREC/PUBREL intermediários pela biblioteca.
   A biblioteca valida IDs/ACKs; não aceitar ACK errado como negócio concluído.
   Enquanto publica, erro/EOF/timeout deve falhar com frase
   'publicacao interrompida; resultado desconhecido, nao reenviada'. Não reconectar
   nem chamar publish novamente nessa fase. Depois terminal, DISCONNECT e saída.
4. Sub enfileira filtro/QoS; SUBACK deve conter exatamente um Success com QoS
   solicitado (redução recusada por compatibilidade). Antes SUBACK pode haver
   matching PUBLISH: exibir/count limitado, mas validar SUBACK antes encerrar.
   QoS1/2 ACKs são automáticos rumqttc, continuar poll necessário para progresso.
5. Sub reconnect: se session_present e assinatura previamente confirmada, preservar;
   se sessão ausente ou sem confirmação anterior, remover só Subscribe pendente
   antes de pedir Subscribe novamente. Não duplicar pending requests da biblioteca.
   clean da biblioteca descarta pending quando CONNACK não retoma sessão.
6. Recepção QoS2 PUBLISH inicia espera limitada8s para PUBREL/PUBCOMP, mesmo com
   --count satisfeito. Em queda durante esse handshake, falhar com frase
   'recepcao QoS2 interrompida; retomada do handshake nao suportada'. Não retentar:
   rumqttc0.25.1 limpa estado incoming QoS2 e rejeita PUBREL após clean.
   PUBCOMP é flushado pela biblioteca antes de evento PUBREL entregue.
7. Retentar apenas transporte transitório: NetworkTimeout, FlushTimeout,
   MqttState::ConnectionAborted/AwaitPingResp e I/O connection refused/reset/
   aborted/EOF/brokenpipe/timedout/notconnected/wouldblock incluindo Io aninhado
   em MqttState/Deserialization. TLS/auth/unsolicited/malformed/NotConnAck não retry.
   Após cada erro elegível consumir uma tentativa total, aguardar100*2^k ms,
   cap5000ms. Se sem orçamento, erro 'limite de reconexao esgotado'. Logs stderr:
   'Reconexao {usadas}/{total} em {ms}ms'. --reconnect-attempts0 encerra no primeiro erro.
8. SUBACK e DISCONNECT deadline8s; idle90s por evento (ping também é atividade).
   QoS2 deadline fixo não renovado por ping. Nunca loop ilimitado de reconnect.
9. Ctrl+C cancela polling/conexão/backoff/ACK/idle. Se rede live, descartar requests
   de aplicação sem publicar durante cancelamento e tentar DISCONNECT por até1s;
   se indisponível, sair sem reconnect. Log stderr, exit0. Em fluxo normal,
   DISCONNECT até8s. Não prometer Will suprimido quando link já indisponível.

## Saída e compatibilidade C26

text padrão: payload UTF-8 lossy com Debug escaping terminal; linha:
topico={topic:?} qos={n} retain={bool} mensagem={text:?}.
Status assinatura ativa em stdout texto e stderr JSON.
Pub textoQoS0: 'PUBLISH QoS 0 enviado; o protocolo nao confirma entrega ao assinante.'
Pub textoQoS1/2: 'Publicacao confirmada pelo broker em QoS {n}.'
Nunca interpretar essas confirmações de protocolo como sucesso de negócio.

jsonl stdout: somente objetos de dados, um por linha sem status/segredos.
PUBLISH recebido: {"event":"publish","topic":string,"qos":0|1|2,
"retain":boolean,"payload_bytes":[inteiros0..255]}. Preserve array vazio,
todos bytes inválidos UTF-8, NUL/newline; string topic escapes quotes, backslash
e todos controlesU+0000..U+001F (Unicode restante válido sem normalização).
Pub terminal: {"event":"publish_complete","qos":n,"confirmation":"sent"}
paraQoS0, broker_protocol_ack paraQoS1/2. Diagnósticos/state/retries sóstderr.
Sem timestamps/novos metadados neste checkpoint; não trocar schema silenciosamente.

## Estrutura nova sugerida (não usar fontes prontos)

main.rs orchestration/help/exit; args.rs parser/validações; security.rs senha/PEM/TLS;
format.rs apresentação JSON/texto; engine.rs rumqttc/estados/backoff/cancelamento.
Pode escolher outros módulos; mesmos contratos públicos de execução.
Não usar patch/copy do src/ de produção como implementação da reconstrução.

## Aceite e validação planejados antes dos fontes

Destino validation/c27-bootstrap deve não existir; criar vazio e registrar
hashes/configuração. Escrever fontes novos, documentar insumos e diferenças.
Executar cargo fmt --all -- --check, cargo test --locked --offline,
cargo clippy --locked --offline --all-targets -- -D warnings, cargo build
--locked --offline dentro do destino. Usar target próprio; nenhum crate broker.

Reutilizar declaradamente tests/wire.rs C26: nove cenários (pub/subQoS0/1/2,
reconnect sessão presente/ausente e ID estável, budget/EOF, auth/peer payload,
count antesSUBACK, pub sem replay, recepçãoQoS2 interrompida) em portas efêmeras.
Escrever unitários novos cobrindo CLI/UTF-8/quota/flags incompatíveis, JSON allbytes,
PEM bounded/falha fechada, policy retry/cancelamento. Verificar fontes novas com
hashes diferentes; igualdade de poucos trechos de APIs não prova cópia/independência.
Rodar gates/inventário do cliente atual somente se alterado ou para checkpoint;
confirmar seus hashes src/ iguais antes/depois. Validar links ativos em índice/matriz.
Gates Linux/MSRV1.88, TLS real/Will/restart/interoperabilidade externa são distintos
e BLOCKED quando indisponíveis. WSL estado anterior: status0, execução timeout10s.
Não instalar software ou executar broker compartilhado para ocultar lacunas.

Critério de conclusão: contrato independente escrito + reconstrução transparente
compilável/testada nos contratos Windows disponíveis, diferenças/limites explícitos
em registro PASS/FAIL/BLOCKED. Não afirmar todos cenários reais/equivalência integral.
Checkpoint C27/C26 antes de mudanças de versão/publicação e antes de implementar C28.

## Revisao1.1 - coordenacao Unix antes dos gates adicionais

Tarefa do broker informou Ubuntu-26.04 operacional/Rust1.99.0 e broker pausado;
autoriza usar binario existente do broker SOMENTE em fixtures com estado/portas
proprios, sem editar repo broker. Ubuntu26 confirmado por preflight, apenas
stable instalado (nao1.88). Executar gates Linux --locked --offline para C26 e
C27 com target dirs isolados; sem instalar/cachefetch/upgrades. Se binario ou
cache faltarem, BLOCKED. Integracao opcional harnessWill com --client explicito,
state temporario e portas efemeras; ler harness antes de executar. Nao usar
resultado baseline26c1019 do pai para afirmar C26 validado. Registrar resultados
atuais e preservar historico dos bloqueios anterioresC26. Nao editar fonteprod.

## Revisao1.2 - criterios integracao isolada antes do harness novo

Binario broker existente localizado por indiceP43 do pai (somente leitura):
/mnt/c/Users/licod/Documents/Codex/2026-10-07/task/p43-broker-bin.
Usar verify_last_will.py existente, --client binario C26 ou C27 explicitamente,
TMPDIR sob target proprio. Planejar fixture adicional validation/C27-integration.py
com --broker e --client explicitos: JSON binario/tópico1024UTF8/retain,QoS0/1/2;
restart do brokerproprio e sessao presente/ausente com retry e ID estavel;
SIGINT live sem Will e durante backoff, sem processos compartilhados.
Fixture TLS propria (Pythonssl/OpenSSL ja instalado, sem brokersecure real):
CA/SAN/mTLS valido e CA/SAN negados sem retry/semCONNECT; segredo fixture0600
em tempdirprivado, nenhum secret em log/Git. Sem baixar/instalar. Todos processos
com deadline, ports loopbackefemeros, state temporario, cleanup somente testowned.
Esses testes comprovam cenarios especificos, nao interoperabilidade Mosquitto,
MSRV1.88 ou brokerTLSprod. Registrar contagens separadas para cada binario.

## Revisao1.3 - ajuste de validacao antes das alteracoes

Rust1.99 Clippy apontou assert_is_empty em3asserts das fixtures C26; trocar
apenas por assert_eq(output.stdout, [] as [u8;0]) em tests/wire.rs e na copia
C27. Nao alterar srcprod nem criterio vazio. Inputs iniciais permanecem;
registrar hash anterior/novo da fixture em approved_asset_changes.json e
adaptar check_provenance para verificar essa unica mudanca explicitada.

Fixture TLS tentou password-file0600 em /mnt/d viaTMPDIR e o cliente recusou
antes deCONNECT; DrvFS nao refletiu modo0600. Usar /tmp Linux nativo somente
para certificados/segredo temporarios700/600; manter validacao de owner/0600,
nao alterar cliente nem oferecer bypass. Redigir assert sem imprimir segredo
mesmo se vazar, registrar fase de falha de fixture sem expor bytes sensiveis.
Repetir testes afetados/gates Clippy1.99/build e integracaoTLS isolada.
