# xmqr-client

## Candidata 0.42.0 — correções da auditoria MQTT 5

C140–C142 corrigem as cinco falhas obrigatórias P232: publicação acima de
Maximum QoS, retained indisponível, aceitação de sessão sem estado local,
entrega duplicada QoS 2 e ausência de PUBCOMP para PUBREL repetido. As quatro
recomendações de validação recebida permanecem abertas; isso não é certificação.
Os prompts e [registro de aceite](prompts/registros/C143-five-audit-fixes.md)
preservam os resultados anteriores. A candidata também inclui C139:
`mqtt-bridge 0.4.0`, origem MQTT 5/QoS 2 com journal privado e destino MQTT 3.1.1.
[Mudanças e migração](docs/releases/0.42.0.md). Distribuição só após gates.

Um processo novo MQTT 5 sem estado local recusa Session Present=1, mesmo
com `--clean-start false`. Para operação sem retomada, use Clean Start padrão.
Para retomar no Linux/WSL, use outbox ou manifesto/journal válidos descritos
abaixo, com diretório privado 0700 e a mesma identidade/configuração. Não apague
estado pendente para contornar uma recusa. Capacidade insuficiente no CONNACK
falha antes de PUBLISH, sem reduzir QoS nem remover a outbox; um PUBREL já
persistido continua concluindo sua transação sem republicar. Os exemplos
históricos C66–C99 abaixo que retomam apenas por ID/flags não substituem esse
contrato atual e são recusados se o broker declarar sessão anterior.

A prerelease 0.41.0 do client inclui `mqtt-bridge 0.3.0` com origem MQTT 5
opt-in em QoS 0/1 e Retain As Published, preservando replay e publicações
retained vivas no destino MQTT 3.1.1. C137 confirma PUBACK na origem somente
após PUBACK no destino para QoS 1; a janela de crash admite duplicação.
[Guia](docs/bridge-one-way.md), [C136](prompts/registros/C136-bridge-mqtt5-retain-as-published.md)
e [C137](prompts/registros/C137-bridge-mqtt5-qos1-retain.md).
[Release](https://github.com/licodevone/xmqr-client/releases/tag/v0.41.0),
[nota](docs/releases/0.41.0.md) e [registro C138](prompts/registros/C138-release-0.41.0.md).
A [fase 6](https://github.com/licodevone/xmqr/blob/main/docs/phase6-acceptance.md)
foi aceita no recorte finito do broker 0.43.0; a fase 8 segue aberta.

A prerelease experimental 0.40.0 distribui C134: bridge MQTT 3.1.1 QoS 2 com journal
privado da origem e retomada de sessão nas duas pontas. O binário da bridge
passa a `mqtt-bridge 0.2.0`; a release 0.39.0 continua limitada a QoS 0/1.
Veja a [release](https://github.com/licodevone/xmqr-client/releases/tag/v0.40.0),
[nota](docs/releases/0.40.0.md) e [registro C135](prompts/registros/C135-release-0.40.0.md).
Na data desta release histórica, as fases 6 e 8 ainda não tinham aceite.

A fonte C134 acrescenta bridge MQTT 3.1.1 QoS 2 com journal privado da
origem, retomada condicionada à sessão e ensaios de queda. A
[CI da fonte exata](https://github.com/licodevone/xmqr-client/actions/runs/39158793553)
passou; esta fatia não integra a release 0.39.0 e foi distribuída na 0.40.0. Veja o
[guia da bridge](docs/bridge-one-way.md) e o
[registro C134](prompts/registros/C134-bridge-qos2-journal.md).

## Prerelease experimental 0.39.0 — C133

Esta versão reúne o [journal inbound MQTT 5](docs/mqtt5-inbound-journal.md)
publicado no client 0.38.0 e o sidecar `mqtt-bridge` C125–C128 no `.deb` e na
imagem. A bridge TLS/mTLS unidirecional oferece QoS 0/1 e serviço opcional,
sem início automático. [Guia, configuração e limites](docs/bridge-one-way.md).
O journal pode duplicar saída na janela stdout→commit; ACK não comprova
processamento de negócio. Preserve seus diretórios privados em downgrade e
consulte a [nota 0.38.0](docs/releases/0.38.0.md). A fase 6 global, a fase 8
global e a homologação Windows não são declaradas concluídas.

[Release 0.39.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.39.0),
[nota de instalação e rollback](docs/releases/0.39.0.md) e
[registro C133](prompts/registros/C133-bridge-package-039.md). O client 0.38.0
anterior contém somente `mqtt-client`, sem a bridge empacotada.

A prerelease experimental 0.36.0 reúne C122/C123: outbox QoS 2 e manifesto
de assinatura MQTT 5 por WSS mTLS, com URL canônica como identidade e retomada
condicionada a Session Present. [Registros C122](prompts/registros/C122-wss-qos2-outbox.md)
e [C123](prompts/registros/C123-wss-durable-subscription.md). A release 0.35.0
não inclui essas entregas.
[Release 0.36.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.36.0),
[nota](docs/releases/0.36.0.md) e [registro C124](prompts/registros/C124-release-0.36.0.md).

A prerelease experimental 0.35.0 reúne C119 e permite outbox MQTT 5
QoS 1 por WSS mTLS com URL canônica como identidade do destino; TCP mantém
seus journals antigos. Na 0.35.0, WS aberto, QoS 2 e manifesto de assinatura por URL
são recusados. C120 validou reconexão com Session Present e DUP após
PUBACK perdido em fixture WSS mTLS. [Registros C119](prompts/registros/C119-wss-qos1-outbox.md)
e [C120](prompts/registros/C120-wss-qos1-outbox-recovery.md).
[Release 0.35.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.35.0),
[nota](docs/releases/0.35.0.md) e [registro C121](prompts/registros/C121-release-0.35.0.md).

A prerelease experimental 0.34.0 inclui C117 e permite
`--allow-remote-wss --ws-url wss://broker.exemplo:8443/mqtt` em mTLS seguro.
A flag é obrigatória para URL não loopback; CA/SAN, certificado cliente,
credenciais MQTT e ACL continuam necessários. Não há fallback para WS aberto,
nem navegador ou estado durável por URL. [Prompt C117](prompts/clients/117-remote-wss-mtls-client.md)
e [aceite parcial](prompts/registros/C117-remote-wss-mtls-client.md).
[Release](https://github.com/licodevone/xmqr-client/releases/tag/v0.34.0),
[nota](docs/releases/0.34.0.md) e [registro C118](prompts/registros/C118-release-0.34.0.md).

C116, incluído na release 0.34.0, permite `ws_url = "wss://127.0.0.1:PORTA/mqtt"`
em `[connection]` do perfil TOML v1. O perfil reutiliza CA/certificado/chave,
usuário e arquivo privado de senha; `--ws-url` explícito tem precedência.
`host`/`port` efetivos junto da URL são recusados. WS aberto por perfil,
TLS-senha, estado durável por URL e navegador continuam fora
do escopo. [Prompt C116](prompts/clients/116-wss-mtls-profile.md) e
[registro](prompts/registros/C116-wss-mtls-profile.md).

A prerelease experimental 0.33.0 reúne C114: `--ws-url wss://127.0.0.1:PORTA/mqtt`
permite MQTT 3.1.1/5 via WSS com CA/SAN, certificado de cliente e credenciais
MQTT do modo seguro. O broker P212 fornece o listener mTLS local na release 0.40.0.
Perfis, estado durável sobre URL e acesso remoto continuam pendentes. A
prerelease 0.32.0 abaixo ainda não contém C114.
[Prompt C114](prompts/clients/114-mtls-websocket-client.md) e
[registro](prompts/registros/C114-mtls-websocket-client.md).
[Release](https://github.com/licodevone/xmqr-client/releases/tag/v0.33.0),
[nota](docs/releases/0.33.0.md) e [registro C115](prompts/registros/C115-release-0.33.0.md).

A prerelease experimental 0.32.0 distribui C112: `--ws-url ws://127.0.0.1:PORTA/CAMINHO` (ou `[::1]`)
para `pub`/`sub` MQTT 3.1.1 e 5 com `--open-lab` explícito. É um recorte
experimental em loopback: WSS, perfis e manifestos duráveis ainda não são
suportados. O broker XMQR 0.39.0 fornece WS local. Exemplo: `mqtt-client sub --open-lab
--ws-url ws://127.0.0.1:8083/mqtt --topic 'lab/#' --count 1`.
[Prompt C112](prompts/clients/112-loopback-websocket-transport.md) e
[registro](prompts/registros/C112-loopback-websocket-transport.md).
[Release](https://github.com/licodevone/xmqr-client/releases/tag/v0.32.0),
[nota](docs/releases/0.32.0.md) e [registro C113](prompts/registros/C113-release-0.32.0.md).

A prerelease experimental 0.31.0 distribui C110: com manifesto de assinatura
MQTT 5 no Linux/WSL, o ACK QoS 1/2 ocorre após a saída; erro de stdout deixa
o broker reenviar. Ainda não há journal inbound nem processamento exatamente
uma vez. [Release](https://github.com/licodevone/xmqr-client/releases/tag/v0.31.0),
[nota](docs/releases/0.31.0.md) e [registro C111](prompts/registros/C111-release-0.31.0.md).
Fases globais abertas.

A release experimental 0.30.0 reúne C107/C108 sobre a release 0.29.0.
C107 retoma uma publicação QoS 2 por estágio durável; C108 retoma a declaração
de uma assinatura MQTT 5 sem reenviar SUBSCRIBE quando o broker confirma a
sessão. A versão 0.29.0 publicada reúne C104/C105. C105 acrescenta uma outbox
experimental para uma publicação MQTT 5 QoS 1 por processo em Linux/WSL.
Crie um diretório privado com `mkdir -m 700 ~/xmqr-outbox` e use, por exemplo,
`mqtt-client pub --open-lab --protocol-version 5 --host 127.0.0.1 --port 1883
--topic lab/temperatura --message 23.5 --qos 1 --client-id sensor-a
--clean-start false --session-expiry-seconds 60 --outbox-dir ~/xmqr-outbox`.
Se a confirmação for incerta, repita **o mesmo comando** antes de a sessão
expirar. A retomada envia DUP com o mesmo Packet Identifier somente quando o
broker confirma Session Present; caso contrário, a outbox fica preservada
para investigação, sem publicação automática. O diretório deve existir,
pertencer ao usuário e ter modo 0700; os arquivos usam 0600 e contêm o
payload em claro. QoS 1 admite duplicatas; PUBACK não confirma que um
assinante ou destino externo processou os dados. [Prompt C105](prompts/clients/105-durable-qos1-outbox-mqtt5.md).

C107 amplia a mesma opção `--outbox-dir` para `--qos 2` no Linux/WSL. Use um
diretório privado por publicação, Client Identifier fixo, `--clean-start false`
e Session Expiry positivo. Após queda antes do PUBREC, a retomada envia
PUBLISH com DUP e o mesmo Packet Identifier; após PUBREC positivo, o journal
registra a fase PUBREL antes de transmiti-lo e a retomada envia apenas PUBREL.
Sem Session Present, preserva o arquivo e falha. Repita o comando exatamente;
não compartilhe o Client Identifier com outro processo. O journal QoS 1 v1
permanece legível, e o journal QoS 2 é separado. QoS 2 confirma a troca com o
broker, não o processamento por assinantes, Node-RED ou Google Sheets.
Múltiplas publicações pendentes ainda não têm estado local durável.
[Prompt C107](prompts/clients/107-durable-qos2-outbox-mqtt5.md).

C108 acrescenta `sub --subscription-state-dir DIRETÓRIO` no Linux/WSL para
uma declaração de assinatura MQTT 5 contínua. Exige Client Identifier fixo,
`--clean-start false`, Session Expiry positivo e não aceita `--count`. O
diretório deve existir, pertencer ao usuário e ter modo 0700. Depois de
SUBACK, um manifesto 0600 registra a declaração como ativa; ao repetir o
mesmo comando, o client exige Session Present=1 e escuta a assinatura já
guardada no broker sem reenviar SUBSCRIBE. Se o broker perdeu a sessão ou
filtros/opções mudaram, falha preservando o manifesto. Se o SUBACK anterior
foi incerto, repete SUBSCRIBE. Esse manifesto não guarda mensagens recebidas:
queda após confirmação MQTT pode perder a saída exibida, e retomada não
promete processamento exatamente uma vez. [Prompt C108](prompts/clients/108-durable-subscription-resume-mqtt5.md).

Após a release 0.30.0, C110 muda a assinatura persistente para confirmar
PUBLISH QoS 1/2 somente depois de escrever e dar flush na saída. Uma falha
de stdout não envia PUBACK/PUBREC; o broker pode reenviar na sessão. Mensagens
filtradas também são confirmadas. Isso reduz a janela de perda anterior, mas
o manifesto não guarda mensagens recebidas: queda após flush e antes do ACK
pode duplicar a saída, e stdout não confirma gravação no consumidor. O modo
sem `--subscription-state-dir` mantém o comportamento anterior. [Prompt C110](prompts/clients/110-durable-subscription-ack-after-output-mqtt5.md).

C104 acrescenta
`--disconnect-session-expiry-seconds 0..4294967294|never` a `pub --protocol-version 5`
e `sub --protocol-version 5 --count N`. A opção envia Session Expiry Interval no
DISCONNECT normal; sem ela, o pacote e o prazo do CONNECT permanecem como antes.
Se o CONNECT anuncia zero, somente zero pode ser enviado no DISCONNECT.
Fora da outbox C105/C107, o client não persiste estado QoS de mensagens;
a fase 6B continua aberta.
[Prompt C104](prompts/clients/104-disconnect-session-expiry-mqtt5.md).
[Release 0.30.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.30.0),
[nota](docs/releases/0.30.0.md) e [aceite C109](prompts/registros/C109-release-0.30.0.md).
[Release 0.29.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.29.0),
[nota](docs/releases/0.29.0.md) e [aceite C106](prompts/registros/C106-release-0.29.0.md).

A release experimental 0.28.0 reúne C101/C102: diagnóstico de DISCONNECT
MQTT 5 e propriedades contextuais de assinatura. C102 permite `sub --protocol-version 5
--subscribe-user-property CHAVE=VALOR` e
`--unsubscribe-user-property CHAVE=VALOR` repetíveis. Cada contexto aceita até
16 pares/2048 bytes codificados, com campos de até 256 bytes; a CLI valida
antes da conexão. A opção de UNSUBSCRIBE exige `--count N`, pois o client só
envia o cancelamento após receber essa quantidade de mensagens. Os pares
mantêm ordem e podem acompanhar `--subscription-identifier`. O broker
0.33.0/P180 é o par distribuído; broker 0.32.0 ainda não contém P180.
[Release 0.28.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.28.0),
[nota](docs/releases/0.28.0.md) e [aceite C103](prompts/registros/C103-release-0.28.0.md).

Após 0.27.0, C101 mostra o código do DISCONNECT MQTT 5 enviado pelo broker
sem reproduzir Reason String remota no terminal. O client continua encerrando
com erro; nenhuma reconexão automática ou release isolada foi acrescentada.
[Registro C101](prompts/registros/C101-server-disconnect-diagnostic-mqtt5.md).

A release experimental 0.27.0 distribui C97–C99: peça um ID ao broker MQTT 5, guarde o
valor mostrado em stderr e reconecte manualmente com `--client-id ID
--clean-start false`. Para preservar a sessão no broker, defina
`--session-expiry-seconds 60` na criação e na reconexão. A CLI não guarda seu
estado QoS local. [Release 0.27.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.27.0) e [aceite C100](prompts/registros/C100-release-0.27.0.md).

A versão 0.27.0 acrescenta C99: para criar uma sessão durável com ID
atribuído pelo broker, use `--protocol-version 5 --server-assigned-client-id
--session-expiry-seconds 60` na primeira conexão (Clean Start verdadeiro).
Guarde o ID mostrado em stderr e, antes de vencer o prazo, reconecte com
`--client-id ID --clean-start false --session-expiry-seconds 60`. A CLI não
restaura automaticamente o estado QoS local. [Prompt C99](prompts/clients/99-durable-assigned-client-id-mqtt5.md).

A árvore após 0.26.0 também acrescenta C98: `--client-id` aceita 1..254
caracteres ASCII alfanuméricos, `-` ou `_` em MQTT 5, suficientes para
reutilizar o ID de 37 caracteres atribuído pelo broker XMQR. MQTT 3.1.1
mantém o limite de 23. Isto não guarda o fluxo QoS da CLI para retomada
automática. [Prompt C98](prompts/clients/98-long-client-id-mqtt5.md).

A árvore após 0.26.0 acrescenta C97: `--server-assigned-client-id` envia
Client Identifier vazio no CONNECT MQTT 5 e exige o ID atribuído em CONNACK.
O ID aparece em stderr; stdout permanece reservado às mensagens. C97 exigia
sessão limpa sem expiração; C99 permite expiração configurada, mantendo Clean
Start verdadeiro e a exclusão de `--client-id` manual na conexão inicial.
O broker P166 é o par. [Prompt C97](prompts/clients/97-assigned-client-id-mqtt5.md).

A release experimental 0.26.0 reúne C94/C95: metadados e Message Expiry
do Will MQTT 5 com validação local e flags próprias, separadas de PUBLISH.
O broker 0.29.0/P163–P164 inicia o prazo do Will quando o publica, mesmo com
Will Delay e reinício. Fases 6A–6D e 7–9 seguem abertas.
[Release 0.26.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.26.0) e [aceite C96](prompts/registros/C96-release-0.26.0.md).

Na árvore após 0.25.0, C95 acrescenta `--will-message-expiry-seconds
0..4294967295` ao Will MQTT 5. Essa opção é distinta de
`--message-expiry-seconds` do PUBLISH e pode ser usada com Will Delay e os
metadados de C94. O broker P164 inicia a contagem na publicação efetiva;
nenhuma dessas opções faz parte da release 0.25.0.
[Aceite C95](prompts/registros/C95-will-message-expiry-mqtt5.md).

Na árvore após 0.25.0, C94 permite combinar `--will-payload-format 0|1`,
`--will-content-type TEXTO`, `--will-response-topic TOPICO`,
`--will-correlation-data-hex HEX` e `--will-user-property CHAVE=VALOR`
repetível com Will MQTT 5 e `--will-delay-seconds`. O client valida limites e
valores antes de conectar, mantém a ordem dos pares e não interpreta as opções
de PUBLISH como opções do Will. O broker P163 é o par de desenvolvimento;
nenhuma dessas opções faz parte da release 0.25.0. Message Expiry do Will
foi adicionado por C95 no marco 0.26.0. [Aceite C94](prompts/registros/C94-will-metadata-mqtt5.md).

A versão experimental publicada 0.25.0 inclui C92: `--reauth-before-io` combinado com
`--protocol-version 5 --auth-method plain` solicita reautenticação `PLAIN`
após o CONNACK sobre TLS e espera AUTH de sucesso antes de pub/sub. A opção
é explícita; por padrão não há AUTH adicional. O client usa uma cópia local
identificada de `rumqttc` 0.25.1 para expor AUTH na mesma conexão e corrigir
o enquadramento desse pacote; [patches e licença](vendor/rumqttc-0.25.1/XMQR-PATCHES.md).
Não há reautenticação periódica nem rotação de senha durante a sessão nesta
fatia. O broker publicado 0.28.0/P160 é o par testado; tags anteriores não
incluem C92/P160. [Release 0.25.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.25.0)
e [aceite C93](prompts/registros/C93-release-0.25.0.md).

O mesmo candidato inclui C91: `mqtt-client pub|sub --protocol-version 5
--auth-method plain --tls-password --ca CA.crt --username USUARIO` usa
autenticação inicial SASL PLAIN sobre TLS, com senha lida de arquivo privado
(`--password-file`) ou prompt. O client exige que o CONNACK confirme `PLAIN`;
o modo convencional continua padrão. Troca AUTH posterior é implementada
somente pela opção explícita C92 acima.
[Prompt C91](prompts/clients/91-enhanced-plain-connect-mqtt5.md).

A versão experimental 0.24.0 inclui C89: valida filtros compartilhados MQTT 5 no
formato `$share/grupo/filtro` e o anúncio de disponibilidade no CONNACK. O
broker XMQR 0.26.0 distribui mensagens entre membros ativos e preserva
associações de sessões persistentes no estado v13 após P153/P154.
Mosquitto 2.0.22 também foi validado em C89.

A versão experimental 0.23.0 reúne C87: permite `sub --protocol-version 5
--subscription-identifier 1..268435455` e exibe
`metadata.subscription_identifiers` em JSONL, inclusive IDs repetidos. O
client recusa o recurso quando o CONNACK anuncia indisponibilidade. O broker
XMQR 0.24.0 ainda não implementa Subscription Identifier; o parceiro 0.25.0
depende dos gates P151. Mosquitto 2.0.22 já foi validado em C87.

A versão experimental 0.22.0 reúne Topic Alias compacto e opções No Local,
Retain As Published e Retain Handling no assinante MQTT 5. Pacotes e imagem
numerada estão na [release 0.22.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.22.0);
[C86](prompts/registros/C86-release-0.22.0.md) registra os gates.

No marco experimental 0.21.0, C80 exibe propriedades PUBLISH MQTT 5 recebidas
em `sub --output jsonl` e C81 permite `pub --protocol-version 5 --repeat` em
série. [Nota de release](docs/releases/0.21.0.md). A fase 6C segue aberta.

No candidato 0.22.0, C83 acrescenta
`pub --protocol-version 5 --repeat 3 --repeat-interval-ms 20
--publish-topic-alias 1` (com `--topic` e `--message`/`--message-file`).
O broker precisa anunciar Topic Alias Maximum >= 1 no CONNACK. A primeira
mensagem estabelece o alias com tópico completo; as seguintes usam tópico
vazio na mesma conexão, com as propriedades PUBLISH preservadas. Se o broker
não anunciar suporte suficiente, a operação falha antes do primeiro envio.
O alias é local à conexão; não há reconexão/replay MQTT 5 automático.

No candidato 0.22.0, C84 acrescenta
`sub --protocol-version 5 --retain-handling 0|1|2`. Zero pede mensagens
retidas no SUBSCRIBE, um apenas para assinatura nova e dois não as pede nesse
momento. `--ignore-retained` e `--retained-only` continuam filtros locais da
saída; não mudam o pacote SUBSCRIBE. O broker P147 aplica a opção por filtro,
inclusive ao retomar uma sessão persistente.

No candidato 0.22.0, C85 acrescenta `--no-local` e
`--retain-as-published` ao assinante MQTT 5. São bits do SUBSCRIBE por filtro,
combináveis com `--retain-handling`; nenhum altera os filtros locais da saída.
O client foi validado com Mosquitto 2.0.22. O broker XMQR implementa os
dois bits em P148, ainda em validação para sua próxima release.

No marco experimental 0.20.0, `mqtt-client pub --protocol-version 5
--payload-format 1 --content-type text/plain` envia as propriedades MQTT 5.
O payload deve ser UTF-8 válido com indicador 1; o client recusa bytes inválidos
antes de conectar.
Em C81, `pub --protocol-version 5 --repeat 3 --repeat-interval-ms 20`
publica uma fonte única três vezes, reaplica as propriedades MQTT 5 e aguarda
o envio/ACK terminal de cada mensagem antes da próxima. O intervalo mantém
a conexão viva; se uma publicação falhar, as já confirmadas ficam visíveis e
o lote restante não é reenviado. Modo linha e retomada local de QoS continuam
pendentes em MQTT 5.
Em C78, `pub --protocol-version 5` aceita `--response-topic reply/t`,
`--correlation-data-hex 00ff07` e `--publish-user-property trace=one` repetível
(até 16 pares e 2048 bytes codificados). A validação ocorre antes da conexão;
o assinante da release 0.20.0 ainda não exibia esses metadados. Veja a
[nota de release](docs/releases/0.20.0.md). A fase 6C global segue aberta.

No marco experimental 0.19.0, `mqtt-client sub --protocol-version 5
--topic-alias-maximum 1` anuncia que aceita um Topic Alias por conexão.
O valor pode ser 0..16; sem a opção, o padrão é zero. O client mostra o tópico
resolvido também nas entregas abreviadas do broker P139. Esta opção é exclusiva
de `sub` MQTT 5. A assinatura não retoma automaticamente após erro de transporte;
uma nova execução estabelece sua própria tabela de aliases. A publicação do
na versão 0.19.0 o publisher ainda não usava Topic Alias.
Veja a [nota de release](docs/releases/0.19.0.md).

O marco experimental 0.18.0 acrescenta `--receive-maximum` e
`--maximum-packet-size` para assinaturas MQTT 5. Veja a
[nota de release](docs/releases/0.18.0.md). O broker parceiro 0.20.0 conserva
o documento durável v8; a fase 6C ainda não está completa.

Em C72, `mqtt-client sub --protocol-version 5 --receive-maximum 1..65535`
anuncia no CONNECT o máximo de publicações QoS1/2 que o assinante aceita em voo
nessa conexão. Sem a opção, a propriedade é omitida. A opção é exclusiva de
`sub` MQTT 5; o broker P134 aplica a quota inclusive a mensagens retidas e
retomadas após reconexão.
Em C73, `--maximum-packet-size 5..65536` limita o tamanho total de cada pacote
MQTT que o assinante aceita. O broker P136 descarta publicações que não cabem;
um valor pequeno demais para SUBACK encerra a conexão.

O marco experimental 0.17.0 acrescenta
`mqtt-client pub --protocol-version 5 --message-expiry-seconds 0..4294967295`
e Will MQTT 5 com `--will-delay-seconds` opcional. Veja a
[nota de release](docs/releases/0.17.0.md). O broker parceiro 0.19.0 grava
estado v8, que exige backup anterior para rollback do broker.

O marco experimental 0.16.0 acrescenta Clean Start e Session Expiry
zero, finita ou infinita ao subconjunto MQTT 5. Veja a
[nota de release](docs/releases/0.16.0.md) antes de atualizar.

O marco experimental 0.15.0 amplia MQTT 5 para QoS2, UNSUBACK, diagnóstico
de recusas e propriedades informativas de CONNECT. Veja a
[nota de release](docs/releases/0.15.0.md); o suporte MQTT 5 integral ainda
está em desenvolvimento.

Desde a [prerelease 0.14.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.14.0), C57/C58 permitem `mqtt-client pub|sub --protocol-version 5`
em QoS0/1 com Clean Start, sem Will/propriedades avançadas.
O padrão segue MQTT 3.1.1; QoS2, lotes e reconexão nessa versão
ainda não estão disponíveis. Exemplo local:
`mqtt-client pub --open-lab --protocol-version 5 --topic teste --message ola`.
`mqtt-client sub --open-lab --protocol-version 5 --topic teste --count 1`.
Ao atingir `--count` em MQTT 5, o client aguarda UNSUBACK de cada filtro antes
de enviar DISCONNECT; recusa ou timeout encerra com erro.
Em C60/C61, recusas CONNACK/PUBACK/SUBACK MQTT 5 exibem
o motivo e o código, por exemplo `BadUserNamePassword (0x86)` e `NotAuthorized (0x87)`. Um PUBACK de sucesso
confirma aceitação pelo broker, sem atestar processamento pela aplicação consumidora.
Na árvore atual, C63 amplia `pub` e `sub --protocol-version 5 --qos 2` para
o ciclo QoS2 completo; `pub` aguarda PUBCOMP e mostra recusas PUBREC/PUBCOMP.
Em C64, use `--request-problem-info 0|1`, `--request-response-info 0|1` ou
`--connect-user-property CHAVE=VALOR` (repetível) com MQTT 5. Cada campo aceita
até 1024 bytes e os pares até 4096 bytes codificados; não passe segredos nessas
opções. O broker P123 valida os pares, sem armazená-los ou devolvê-los.
Em C66, `--clean-start true|false` e `--session-expiry-seconds 0|never`
permitem criar ou retomar uma sessão MQTT 5 com o broker P127. A expiração
`never` mantém assinaturas e entregas QoS pendentes após desconexão/reinício;
`0` remove a sessão ao desconectar. Em C67, valores finitos de 1 a 4294967294
segundos também são aceitos e enviados ao broker P128.
Use um `--client-id` estável para retomar. `--clean-session` permanece exclusivo
de MQTT 3.1.1. Um novo processo do client não restaura os próprios fluxos QoS
em andamento, por isso esta opção não promete retomada completa do lado client.
Com perfil TOML, use
`protocol_version = "5"` ou `MQTT_CLIENT_PROTOCOL_VERSION=5`.

O [marco experimental 0.13.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.13.0) acrescenta `sub --exclude-topic FILTRO` (repetível,
aplicado somente à saída), `pub --repeat N --repeat-interval-ms M`, saídas
`hex`/`raw` em `sub` e `--will-message-file PATH` para Will binário. `raw` exige
`--count 1`; repetição exige `--message` ou `--message-file` e não reenvia
publicações de resultado desconhecido. Veja C52–C55 em `prompts/registros`.

O [marco experimental 0.12.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.12.0)
completa perfis C05: `clean_session`,
`reconnect_attempts` e `total_timeout_seconds` passam a ser reutilizáveis no
TOML v1. Com `--profile`, a precedência é flag CLI > ambiente explícito >
perfil > default. Variáveis `MQTT_CLIENT_HOST`, `MQTT_CLIENT_PORT`,
`MQTT_CLIENT_USERNAME`, `MQTT_CLIENT_PASSWORD_FILE`, `MQTT_CLIENT_ID`,
`MQTT_CLIENT_OUTPUT`, `MQTT_CLIENT_QOS`, `MQTT_CLIENT_KEEPALIVE_SECONDS`,
`MQTT_CLIENT_TIMEOUT_SECONDS`, `MQTT_CLIENT_TOTAL_TIMEOUT_SECONDS`,
`MQTT_CLIENT_RECONNECT_ATTEMPTS` e `MQTT_CLIENT_CLEAN_SESSION` são aceitas,
além de `MQTT_CA_CERT`/`MQTT_DEVICE_CERT`/`MQTT_DEVICE_KEY`. Caminhos do
ambiente são relativos ao cwd; caminhos do perfil, à pasta do arquivo. Erro
de variável explícita recusa a operação sem usar o valor do perfil. O arquivo
de senha mantém a exigência de dono e modo 0600 no Linux.

O marco experimental 0.11.1 acrescenta `--tls-password` (C48/B10) e corrige
o default da imagem Docker (C49). A tag de código 0.11.0 foi substituída antes
da distribuição; consulte o changelog.

O marco experimental 0.10.0 acrescenta perfis de conexão mTLS opt-in (C47).
Exemplo `connection.toml` junto dos certificados e do arquivo de senha:

```toml
version = 1
[connection]
host = "127.0.0.1"
port = 8883
username = "device"
ca = "ca.crt"
cert = "client.crt"
key = "client.key"
password_file = "password"
qos = 1
```

Use `mqtt-client sub --profile connection.toml --topic 'sensor/#'` ou
`mqtt-client pub --profile connection.toml --topic sensor/value --message leitura`.
Paths relativos partem da pasta do perfil; flags explícitas substituem campos
do perfil. Mantenha o arquivo de senha privado (0600 no Linux). O perfil aceita
somente parâmetros de conexão, até 64 KiB; payload, senha inline e modos de
laboratório não são aceitos. A rotação TLS pertence ao broker e é observada
pelo client em novas conexões.

Para WSS mTLS local em `main` após 0.33.0, substitua `host` e `port` por
`ws_url = "wss://127.0.0.1:8084/mqtt"` no mesmo perfil e mantenha os demais
campos. As URLs são validadas como na CLI C114, sem credenciais embutidas.

C48 acrescenta `--tls-password` para usar CA e validar SAN/validade do servidor
sem certificado de cliente. Exige usuário e senha oculta ou arquivo privado.
Rejeita `--cert`, `--key` e variáveis de identidade de dispositivo nesse modo.
O perfil TOML v1 pode usar `tls_password = true` em `[connection]`, sem campos
`cert`/`key`; flags explícitas continuam tendo precedência. O mTLS segue padrão.

C45 acrescenta prazo total monotônico opcional e jitter limitado de reconexão,
separa CONNACK de reconexão do idle e prioriza cancelamentoSIGINT/SIGTERM.
Veja [robustez da sessão](docs/client-robustness.md) e [registroC45](prompts/registros/C45-robustez-reconexao.md).
O marco0.9.0 passou os gates coordenados e está publicado;
veja [C46](prompts/registros/C46-release-fase3.md) e o
[aceite da fase3](https://github.com/licodevone/xmqr/blob/main/docs/phase3-acceptance.md).
MQTT5 completo continua planejado; C57/C58 cobrem publicação e assinatura iniciais.

O client participa do [checklist da fase2](docs/phase2-acceptance.md).
C43 amplia interop externa sem mudar runtime0.8.1; P85/C44 completaram
o aceite físico ESP32 e encerraram a fase2 no escopo validado, sem declarar produção.

Marco de distribuição atual: **0.9.0**, com imagem
`lepinheiro100/xmqr-client:0.9.0`, .deb Ubuntu26amd64 e manifesto/checksums na
[prerelease](https://github.com/licodevone/xmqr-client/releases/tag/v0.9.0).
Acrescenta a robustez C45 e preserva as versões anteriores.
O [plano detalhado de paridade](https://github.com/licodevone/xmqr/blob/main/docs/plano-paridade-mosquitto.md)
está concluído como planejamento, sem implementar fases futuras nesta entrega.

Projeto Rust independente, extraído do cliente XMQR. Pacote `xmqr-client`,
binário compatível `mqtt-client`. A versão **v0.8.0** acrescenta filtros retained
na saída do assinante. O marco **v0.7.0** concluiu a fase1 com timeout, keepalive e limites;
inclui integração systemd e pacote Ubuntu26.04 amd64. Veja [instalação e serviços](docs/ubuntu-systemd.md).
Projeto experimental; consulte as prereleases no GitHub.

## Instalação Ubuntu / WSL

O pacote experimental abaixo é destinado ao Ubuntu 26.04 **amd64**. Ainda não
há repositório APT próprio: baixe o `.deb` e instale-o com o APT.

1. No Windows, abra o Ubuntu pelo PowerShell. Confira o nome da distribuição
   com `wsl --list --verbose` e ajuste o comando se necessário:

```powershell
wsl -d Ubuntu-26.04
```

2. Execute os próximos comandos **no terminal do Ubuntu**. Confirme Ubuntu
   26.04 e arquitetura `amd64`, depois prepare o download:

```bash
cat /etc/os-release
dpkg --print-architecture
sudo apt update
sudo apt install -y wget ca-certificates
mkdir -p ~/xmqr-pacotes
cd ~/xmqr-pacotes
```

3. Baixe o pacote e confira sua integridade:

```bash
wget -O xmqr-client_0.9.0-1_amd64.deb https://github.com/licodevone/xmqr-client/releases/download/v0.9.0/xmqr-client_0.9.0-1_amd64.deb
wget -O xmqr-client_0.9.0-1_amd64.deb.sha256 https://github.com/licodevone/xmqr-client/releases/download/v0.9.0/xmqr-client_0.9.0-1_amd64.deb.sha256
sha256sum -c xmqr-client_0.9.0-1_amd64.deb.sha256
```

Prossiga somente se a verificação mostrar `OK`.

4. Instale e confirme o pacote, o executável e a conta de serviço:

```bash
sudo apt install ./xmqr-client_0.9.0-1_amd64.deb
dpkg-query -W -f='${Status}\n' xmqr-client
mqtt-client --version
id xmqr-client
```

O resultado esperado da consulta é `install ok installed`. O pacote cria
automaticamente o usuário e grupo `xmqr-client`. O executável chama-se
`mqtt-client`; `mqtt-admin` é instalado pelo pacote do broker.

5. Para conectar com TLS/mTLS e senha, configure o broker primeiro e siga o
   [guia de certificados, credenciais e serviços](docs/ubuntu-systemd.md).
   Para a instância `device`, prepare `/etc/xmqr-client/device.conf` e os
   arquivos `ca.crt`, `client.crt`, `client.key` e `password` em
   `/etc/xmqr-client/device/`. A chave e o arquivo de senha devem pertencer a
   `xmqr-client:xmqr-client`, com permissão `0600`. O usuário MQTT e as ACLs
   precisam existir no broker, e o endereço usado deve corresponder ao SAN do
   certificado do servidor. Não coloque senhas na linha de comando ou no Git.

6. Para manter a assinatura como serviço, após configurar a instância:

```bash
sudo systemctl enable --now xmqr-client-sub@device.service
systemctl status xmqr-client-sub@device.service
journalctl -u xmqr-client-sub@device.service -f
```

Use `Ctrl+C` para sair dos logs. O pacote não inicia instâncias automaticamente.
No WSL, habilite o systemd conforme o guia; a instância habilitada inicia quando
a distribuição é iniciada. Para uso manual, consulte `mqtt-client --help` e os
exemplos do guia.

Para atualizar, baixe e verifique o novo pacote e repita
`sudo apt install ./ARQUIVO.deb`. A configuração é preservada, mas os serviços
são parados; revise os arquivos e inicie novamente as instâncias desejadas.

## Compatibilidade Ubuntu

| Ambiente amd64 | Evidência atual |
| --- | --- |
| Ubuntu 26.04 / WSL2 | Pacotes instalados e testes de integração e systemd com TLS/mTLS aprovados. Alvo atual dos pacotes publicados. |
| Ubuntu 24.04 / WSL2 | Os mesmos binários executaram testes de integração; instalação do `.deb` e units nessa versão ainda não validadas. |
| Outras versões Ubuntu, outras distribuições ou arm64 | Não validadas. Não há pacote arm64 publicado nesta etapa. |

Os pacotes requerem `libc6 >= 2.38`, `libgcc-s1 >= 4.2`, `adduser` e
`init-system-helpers`. O Ubuntu 24.04 testado tem glibc 2.39 e executa os
binários. O teste de sessão após reinício recebeu uma mensagem QoS1 repetida na
primeira execução e passou na repetição; a causa ainda precisa ser confirmada.
Não considere essa validação parcial como suporte completo ao 24.04. O gerador
de pacotes permanece restrito ao Ubuntu 26.04 amd64. Não force dependências nem
substitua a glibc do sistema para instalar um pacote.

## Controlar os clients com systemctl

Execute os comandos no terminal Ubuntu/WSL. Configure o broker, certificados,
senha e ACLs antes de iniciar. O nome após `@` identifica a instância: `device`
usa `/etc/xmqr-client/device.conf` e os arquivos em `/etc/xmqr-client/device/`.
Substitua `device` pelo nome de sua instância em todos os comandos.

### Tabela comparativa de comandos

Execute no Ubuntu/WSL. `device` é o nome da instância do client de assinatura;
substitua-o pelo nome configurado em `/etc/xmqr-client/NOME.conf`.

| Ação | Broker | Client de assinatura |
| --- | --- | --- |
| Iniciar | `sudo systemctl start xmqr-broker.service` | `sudo systemctl start xmqr-client-sub@device.service` |
| Ver status | `systemctl status xmqr-broker.service` | `systemctl status xmqr-client-sub@device.service` |
| Parar | `sudo systemctl stop xmqr-broker.service` | `sudo systemctl stop xmqr-client-sub@device.service` |
| Reiniciar | `sudo systemctl restart xmqr-broker.service` | `sudo systemctl restart xmqr-client-sub@device.service` |
| Iniciar agora e automaticamente | `sudo systemctl enable --now xmqr-broker.service` | `sudo systemctl enable --now xmqr-client-sub@device.service` |
| Parar e desativar início automático | `sudo systemctl disable --now xmqr-broker.service` | `sudo systemctl disable --now xmqr-client-sub@device.service` |
| Verificar se está ativo | `systemctl is-active xmqr-broker.service` | `systemctl is-active xmqr-client-sub@device.service` |
| Verificar início automático | `systemctl is-enabled xmqr-broker.service` | `systemctl is-enabled xmqr-client-sub@device.service` |

### Client de assinatura: processo contínuo

| Objetivo | Comando | Efeito |
| --- | --- | --- |
| Iniciar | `sudo systemctl start xmqr-client-sub@device.service` | Inicia a assinatura agora. |
| Consultar status | `systemctl status xmqr-client-sub@device.service` | Mostra estado, processo e registros recentes. |
| Parar | `sudo systemctl stop xmqr-client-sub@device.service` | Solicita encerramento do client. |
| Reiniciar | `sudo systemctl restart xmqr-client-sub@device.service` | Para e inicia novamente, aplicando a configuração. |
| Habilitar início automático | `sudo systemctl enable xmqr-client-sub@device.service` | Habilita para a próxima inicialização, sem iniciar agora. |
| Habilitar e iniciar agora | `sudo systemctl enable --now xmqr-client-sub@device.service` | Combina início automático e início imediato. |
| Desabilitar início automático | `sudo systemctl disable xmqr-client-sub@device.service` | Remove o início automático, sem parar agora. |
| Desabilitar e parar | `sudo systemctl disable --now xmqr-client-sub@device.service` | Remove o início automático e para agora. |
| Consultar atividade | `systemctl is-active xmqr-client-sub@device.service` | Exibe `active`, `inactive` ou `failed`, por exemplo. |
| Consultar início automático | `systemctl is-enabled xmqr-client-sub@device.service` | Exibe `enabled` ou `disabled`, por exemplo. |

`active (running)` indica processo em execução; confira os logs e a entrega de
uma mensagem para confirmar a assinatura. `inactive (dead)` significa parado;
`failed` indica uma falha. Se a configuração da instância estiver ausente, sua
condição de início pode impedir a execução: confira o status e o caminho do
arquivo. Para `Unit ... could not be found`, verifique a instalação com
`dpkg-query -W xmqr-client`.

Instâncias simultâneas precisam de ClientIDs diferentes. Uma assinatura
contínua pode ser mantida enquanto outra instância publica. Consulte o
[controle do broker](../xmqr/README.md#controlar-o-broker-com-systemctl) para
iniciar o servidor primeiro.

### Client de publicação: execução pontual

A unit `xmqr-client-pub@device.service` publica o conteúdo de
`/etc/xmqr-client/device/payload.bin`, com limite de 4096 bytes. Prepare o arquivo
para que o usuário `xmqr-client` consiga lê-lo e configure `TOPIC` como tópico
de publicação, sem os curingas `+` ou `#`. Para manter publicação e assinatura
simultâneas, prefira instâncias distintas, como `publisher` e `subscriber`, com
seus próprios arquivos `.conf`, diretórios e ClientIDs.

```bash
# Executar uma publicação
sudo systemctl start xmqr-client-pub@device.service

# Conferir estado e resultado
systemctl status xmqr-client-pub@device.service --no-pager
journalctl -u xmqr-client-pub@device.service -n 50 --no-pager

# Interromper uma publicação que ainda esteja em execução
sudo systemctl stop xmqr-client-pub@device.service
```

Esse serviço é `oneshot`: termina após a tentativa de publicação e não reinicia
automaticamente. Após sucesso pode aparecer como `inactive (dead)`; confira
nos logs o evento `publish_complete` e seu resultado de protocolo. Para
consultar o código de saída do processo:

```bash
systemctl show xmqr-client-pub@device.service --property=ExecMainStatus --value
```

O código `0` indica saída com sucesso, mas deve ser interpretado junto com os
logs e a confirmação prevista pelo QoS. Não comprova processamento pelo
destinatário. Repetir `start` ou usar `restart` pode publicar a mensagem de novo.
`stop` não desfaz uma mensagem já enviada. A unit de publicação não possui
seção de instalação para `enable` nem timer; use `start` para cada envio.

### Logs dos clients

```bash
# Acompanhar a assinatura em tempo real
journalctl -u xmqr-client-sub@device.service -f

# Consultar as últimas 50 linhas
journalctl -u xmqr-client-sub@device.service -n 50 --no-pager

# Consultar registros desta inicialização do Ubuntu
journalctl -u xmqr-client-sub@device.service -b --no-pager

# Consultar registros recentes da publicação
journalctl -u xmqr-client-pub@device.service --since "10 minutes ago" --no-pager

# Listar as units de clients atualmente carregadas
systemctl list-units --all 'xmqr-client-*'
```

`journalctl` consulta os registros do systemd; `-u` filtra a unit e `-f` mantém
o acompanhamento de novos registros. Não inicia nem para o client. Pressione
**Ctrl+C** para sair dos logs e **q** para sair da tela paginada de status.
Use `--no-pager` para consultar status sem essa tela. Se o acesso aos logs for
negado, execute a consulta com `sudo`.

### Configuração, falhas e WSL

Depois de alterar o `.conf`, senha ou certificados, reinicie a assinatura com
`sudo systemctl restart xmqr-client-sub@device.service`. Os clients não possuem
`ExecReload`: não use `systemctl reload` para aplicar essas alterações. A
publicação lê os arquivos no próximo `start`.

Se editar uma unit ou override, execute `sudo systemctl daemon-reload` e depois
reinicie a instância desejada. `daemon-reload` recarrega as definições do
systemd, sem iniciar ou reiniciar os clients sozinho.

A assinatura pode reiniciar após falhas, com limite de cinco inícios por 120
segundos. Erros de configuração com saída `2` não provocam reinício automático.
Após corrigir o problema, consulte os logs e, se o limite de início foi atingido:

```bash
sudo systemctl reset-failed xmqr-client-sub@device.service
sudo systemctl start xmqr-client-sub@device.service
```

`reset-failed` limpa o estado de falha, sem corrigir a configuração. Uma parada
solicitada por `stop` não provoca reinício automático. Antes de remover o
pacote, desabilite as instâncias de assinatura com `disable --now`.

No WSL, `ps -p 1 -o comm=` deve mostrar `systemd`; consulte o
[guia Ubuntu e serviços](docs/ubuntu-systemd.md) para habilitá-lo. Instâncias
habilitadas iniciam quando a distribuição Ubuntu é iniciada. Os comandos
controlam os serviços dentro dessa distribuição.

## Diagnóstico e confirmação de telemetria — C35

O client diferencia recusas MQTT de autenticação, autorização, versão e ClientID.
Falhas TLS indicam revisão de CA/SAN/validade e certificado/chave; outras falhas
de transporte não são classificadas automaticamente como erro de certificado.
Mensagens não incluem conteúdo recebido, senhas ou detalhes privados de erros.
O comportamento de reconexão permanece limitado; autenticação recusada não
provoca novas tentativas automáticas.

Para confirmar a telemetria do ESP32, use um usuário com ACL de assinatura para
`camara/cold-room-sim-01/telemetria` e um ClientID diferente do dispositivo.
No Ubuntu, o comando abaixo solicita a senha sem exibi-la. Substitua
`USUARIO_AUTORIZADO` pelo usuário cadastrado e confira os caminhos dos arquivos:

```bash
mqtt-client sub \
  --host 127.0.0.1 --port 8883 \
  --topic camara/cold-room-sim-01/telemetria \
  --username USUARIO_AUTORIZADO --client-id telemetry-check \
  --qos 1 --count 5 --output jsonl \
  --ca ~/xmqr-certificados/ca.crt \
  --cert ~/xmqr-certificados/client.crt \
  --key ~/xmqr-certificados/client.key
```

O exemplo é para execução manual; não registra automaticamente dados reais.
Cinco eventos recebidos comprovam entrega ao client, não processamento de negócio.
QoS1 permite duplicatas; para contagem lógica, compare o identificador/sequência
do payload da aplicação. A assinatura QoS1 não aumenta o QoS da publicação.
Não usar o mesmo ClientID do ESP32: isso pode desconectar a conexão existente.

## Compilar a partir do código

```powershell
cd D:\projects\my-project\rapidez-projects/rapidez-broker-client\xmqr-client
cargo build --locked --bin mqtt-client
cargo run --locked -- --help
```

No WSL use `/mnt/d/projects/my-project/rapidez-projects/rapidez-broker-client/xmqr-client`. O broker
independente permanece em `D:/projects/my-project/rapidez-projects/rapidez-broker-client/xmqr`;
`mqtt-admin` pertence ao broker.
Veja [a imagem Docker do cliente](docs/docker.md) para build e montagem de
certificados.
Cliente usa rumqttc: pub/sub QoS0/1/2, retained, sessões e Last Will; TLS/mTLS
com validação de CA/SAN e credenciais. Laboratórios somente loopback.
Tópicos/filtros: até 1024 bytes UTF-8; payload até 4096. Melhorias C26 abaixo.
Senha interativa oculta; `--password-file` somente Linux/WSL, proprietário
atual e modo 0600. Não há persistência local de sessão. Reconexão opcional descrita abaixo.

Gates: `cargo fmt --all -- --check`, `cargo test --locked`,
`cargo clippy --locked --all-targets -- -D warnings`, `cargo build --locked`.
Integração: no broker, execute `scripts/verify_last_will.py --broker CAMINHO
--client CAMINHO_DO_CLIENTE_INDEPENDENTE` em ambiente Unix isolado.
Guia anterior preservado em docs/original-mqtt-client.md como referência:
caminhos de build e links relativos nele pertencem ao repositório original.
Prompts C19-C24 são históricos/propostas, não autorização de implementação.

Toda melhoria deve começar por um prompt com ID/versão, estado, objetivo,
escopo, arquivos, invariantes, aceite e testes. Veja AGENTS.md e C25.
MIT e atribuições originais preservadas. Separação estrutural concluída com gates Windows e backup verificado.
Validações Unix com broker isolado e peer TLS próprio estão no registro C27;
MSRV1.88 exato e interoperabilidade externa ainda não comprovados.


## Marco C26 (incluído na tag v0.2.0)

A etapa C26 preservou o manifest herdado 0.7.0. C28 inicia a numeração
independente 0.3.0, sem reescrever a tag anterior.

`--reconnect-attempts N` aceita 0..10 (padrão 0, falha sem reconectar).
Usa o mesmo EventLoop rumqttc, Client ID e opções de sessão. O orçamento é total
por processo, incluindo falhas antes do CONNACK e quedas após conectar, sem reset.
Desde C45, as bases são 100, 200, 400, 800, 1600, 3200ms e 5s nas demais;
cada espera usa jitter limitado80..100% da base, sem ultrapassar5s;
cada conexão mantém os timeouts da biblioteca e os deadlines do cliente.
TLS, recusa de autenticação e erros de protocolo não são retentados.

Em `sub`, sessão retomada preserva a assinatura; sessão ausente exige SUBSCRIBE
novamente. Use --client-id ID --clean-session false para solicitar sessão estável
entre conexões/processos; a retomada depende de o broker ainda ter essa sessão.
Não há persistência de estado MQTT no disco do cliente. --count continua contando
através das reconexões. Ctrl+C cancela poll/espera e tenta DISCONNECT por até 1s
se o transporte ainda estiver disponível, sem reconectar durante o encerramento.

Em `pub`, reconexão é permitida somente antes de enfileirar PUBLISH. Se a conexão
cair enquanto se aguarda o envio/ACK, o cliente falha com resultado desconhecido,
sem reenviar a publicação. Uma repetição manual pode produzir duplicatas.
QoS0 indica envio; PUBACK/PUBCOMP confirmam apenas o protocolo do broker,
não sucesso de negócio nem processamento pelo assinante.

`--output text|jsonl` mantém texto por padrão. JSON Lines escreve um objeto por
linha em stdout; estados operacionais, reconexões e erros vão para stderr.
Mensagens têm event, topic, qos, retain e payload_bytes (array de
inteiros 0..255, inclusive payload vazio), preservando binários sem conversão UTF-8.
Com MQTT 5, `sub --output jsonl` acrescenta `metadata` quando recebe propriedades
PUBLISH conhecidas: `payload_format_indicator`, `message_expiry_interval`,
`content_type`, `response_topic`, `correlation_data_hex` (bytes em hexadecimal)
e `user_properties` (lista ordenada de pares, inclusive chaves repetidas).
Sem essas propriedades, o campo é omitido. Topic Alias é detalhe da conexão:
o campo `topic` contém o nome resolvido. MQTT 3.1.1 e saídas text/hex/raw
conservam o formato anterior.
Conclusão de pub usa event: "publish_complete" e confirmation igual a sent
(QoS0) ou broker_protocol_ack (QoS1/2). Credenciais não integram esses objetos.

```powershell
cargo run --locked -- sub --open-lab --topic 'test/#' --count 2 --client-id stable-id --clean-session false --reconnect-attempts 3 --output jsonl
```

Validação C26 usa fixtures MQTT próprias em loopback/portas efêmeras, sem executar
o broker vizinho ou Mosquitto. Isso não substitui os gates de integração Unix/TLS
com broker independente; a execução isolada posterior está registrada em C27.

Limitação verificada: rumqttc 0.25.1 limpa o estado de recepção QoS2 ao perder
a conexão. Se a queda ocorrer entre PUBLISH recebido e conclusão PUBREL/PUBCOMP,
o cliente falha explicitamente, sem reconectar nessa fase. O handshake recebido
tem deadline padrão de 8s, configurável por `--timeout-seconds`; `--count` aguarda sua conclusão antes de DISCONNECT.

Prompts: veja [índice e ordem](prompts/README.md) e [matriz de cobertura](prompts/COBERTURA.md).
Os históricos não constituem um bootstrap independente; use o contrato C27.

## Checkpoint C27

[Contrato de bootstrap](prompts/clients/27-bootstrap-independente.md) cobre o cliente
até C26. [Reconstrução de validação](validation/c27-bootstrap/README.md) tem fontes
novos, insumos reutilizados explicitamente e comparação por contratos, sem substituir
este cliente ou prometer identidade de bits. [Resultados e limitações](prompts/registros/C27-bootstrap.md).

O binário cliente é Rust; Python auxilia testes, inventário e conferência de hashes,
não é um cliente alternativo nem requisito para executar mqtt-client. TOML/JSON são
configurações/metadados; Markdown é documentação. Comandos usam PowerShell/bash.

## Marco C28 — incluído na tag v0.3.0

[Prompt C28 rev.2.0](prompts/clients/28-payload-binario-planejado.md) e
[registro de validação](prompts/registros/C28-payload-binario.md).
Use exatamente uma fonte em pub: --message TEXTO, --message-file CAMINHO ou
--stdin. Arquivo deve ser regular; erros não exibem caminhos ou bytes recebidos.
O modo bruto preserva todos os bytes (BOM, NUL, UTF-8 inválido e CRLF), incluindo
payload vazio, até 4096 bytes; aguarda EOF. Não converte a entrada em texto.

--line-mode, somente com arquivo/stdin, publica uma mensagem por linha: remove
LF e um CR imediatamente anterior, preserva CR isolado, BOM e bytes binários.
Linha vazia publica payload vazio; EOF publica última linha sem LF; entrada
vazia não conecta. LF final não cria mensagem extra. O limite de4096 bytes vale
por payload após esse enquadramento. Retain vale para cada linha: uma linha
vazia com --retain true apaga o retained desse tópico.

Cada publicação aguarda envio QoS0 ou ACK terminal QoS1/2 antes da seguinte.
Essa regra também vale para `--repeat` MQTT 5 em C81, com uma mensagem por vez.
No modo por linha, cada conclusão escreve o mesmo formato text/JSONL de C26.
Uma falha posterior retorna erro e mantém as conclusões anteriores; não há
rollback. Depois do primeiro PUBLISH não reconecta nem reenvia o lote.
ACK confirma protocolo, não processamento pelo assinante ou sucesso de negócio.

A primeira entrada é validada antes de credenciais/rede. Cada espera de entrada
tem limite fixo de8s, sem renovação por eventos MQTT. Ctrl+C cancela a espera e
tenta DISCONNECT no link ativo por até1s. A leitura nativa bloqueada não impede
o encerramento do processo; fila limitada e publicação serial aplicam backpressure.
Senha oculta usa console/TTY separado; --password-file preserva proteção Linux0600.

~~~powershell
cargo run --locked -- --version
cargo run --locked -- pub --open-lab --topic 'test/binary' --message-file 'payload.bin' --qos 1 --output jsonl
cargo run --locked -- pub --open-lab --topic 'test/lines' --message-file 'lines.bin' --line-mode --qos 2
~~~

Para stdin binário use um pipe que preserve bytes; comandos de texto do shell
podem recodificar a entrada antes de o cliente recebê-la. --version imprime
a versão atual sem ler entrada/senha ou abrir conexão.

Gates Windows/Linux e18 cenários isolados (C28+C27+Will) passaram; MSRV1.88
exato não instalado e Mosquitto/broker TLS completo continuam não comprovados.
O bootstrap C27 é evidência congelada do contrato da tag0.2; não é reconstrução
independente da nova versão0.3. Não executar seu checker antigo para declarar
igualdade com os fontes após C28.

## Encerramento de serviço

SIGTERM/SIGINT usam DISCONNECT limitado a1s no link disponível. A unit de
publicação não reinicia; a de assinatura tem reinícios limitados. Atualizações
param instâncias; reinicie explicitamente após revisar configuração.

## Filtros retained na assinatura — C41

`sub --ignore-retained` omite mensagens recebidas com RETAIN=true;
`sub --retained-only` exibe somente essas mensagens. Sem flag, todas são exibidas.
As opções são exclusivas, não aceitam valor e não valem para `pub`.

```sh
mqtt-client sub --open-lab --topic 'sensor/#' --ignore-retained --count 1 --output jsonl
mqtt-client sub --open-lab --topic 'sensor/#' --retained-only --count 1 --output jsonl
```

`--count` conta somente mensagens exibidas. Mensagens filtradas continuam sendo
recebidas e confirmadas em QoS1/2; o filtro não muda a assinatura nem a ACL.
RETAIN é a flag recebida do broker: o replay de retained normalmente a usa,
enquanto publicações ao vivo chegam com RETAIN=false mesmo quando o publicador
usou `--retain true`. O filtro não deduz quais tópicos têm retained armazenado.
Text e JSONL seguem a mesma regra; reconexão, cancelamento e timeouts permanecem.

## Publicação verificada

Prerelease da fase1: [v0.7.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.7.0),
com deb Ubuntu26 amd64 e checksum. Aceite em
[registro C39](prompts/registros/C39-release-fase1.md).


## Evolução e paridade

Veja a [matriz Mosquitto](docs/mosquitto-parity.md), com lacunas, evidências e
próximos incrementos. C36 é documental; C37 implementa timeout/keepalive com fixtures locais.

## Timeout e keepalive — C37

`--keepalive-seconds N` aceita 1..65535, padrão 30, enviado em CONNECT e
preservado na reconexão. Zero, embora permitido pelo MQTT, fica fora deste contrato.
`--timeout-seconds N` aceita 1..300, padrão 8. Limita cada poll/tentativa antes
do CONNACK e os deadlines de SUBACK, envio/ACK terminal de PUBLISH, conclusão
do QoS2 recebido e DISCONNECT normal. Não limita o comando inteiro, espera por
mensagens de aplicação ou backoff de reconexão. O orçamento de retry permanece
inalterado; expirar o deadline externo não cria nova tentativa.

A assinatura mantém sua inatividade de 90s, entrada de payload mantém 8s e
cancelamento mantém DISCONNECT de até 1s. Publicação interrompida tem resultado
desconhecido e não é reenviada. A opção não altera os timeouts internos do rumqttc.

C45 corrige a renovação do idle: após SUBACK, somente PUBLISH recebido reinicia
90s; PINGRESP/eventos de saída não mascaram ausência de mensagens. Na reconexão,
CONNACK usa protocolo configurado, não idle90s. SUBACK/idle incluem backoff.

`--total-timeout-seconds N` aceita1..86400, opcional; padrão sem prazo total.
Usa uma deadline monotônica desde session::run após a primeira entrada de payload
e senha, incluindo TLS/rede/backoff/protocolo/mensagens/entrada posterior e
DISCONNECT normal. Nunca renova; não limita a senha interativa nem a entrada
inicial (8s). Expiração sai com erro, sem reenvio nem período adicional.
SIGINT/SIGTERM já pendentes são priorizados sobre poll e nova tentativa.

Exemplo local: `mqtt-client sub --open-lab --topic test/# --keepalive-seconds 45 --timeout-seconds 3`.
Veja [registro C37](prompts/registros/C37-timeout-keepalive.md).

## Limites da fase1 — C38

Payload de publicação até4096 bytes; tópico/filtro até1024 bytes UTF-8. O SUBSCRIBE de saída inclui cabeçalho e corpo no teto65536 bytes:65536 aceito,65537 recusado antes de rede.

Na recepção, rumqttc0.25.1 limita Remaining Length, não o frame completo. O client configura65531 bytes de corpo e reserva o cabeçalho máximo de5bytes; assim nenhum frame aceito excede65536 bytes, inclusive comprimento codificado sem a forma mínima. Essa proteção pode recusar um frame canônico de65536 bytes por1byte. Não afeta as publicações suportadas pelo broker (payload4096/tópico1024). Não há promessa de fronteira inclusiva idêntica de entrada/saída; uma necessidade externa dessa fronteira exige escopo próprio para o codec.

SUBACK recusado orienta conferir ACL/limites; MQTT3.1.1 não informa causa específica. Uma queda de transporte não comprova quota, e publicação interrompida mantém resultado desconhecido sem reenvio. PUBACK/PUBCOMP confirmam protocolo, sem reason de quota.

Integração real isolada (Unix, sem usar serviços/estado instalado):

```bash
python3 scripts/verify_phase1.py --broker ../xmqr/target/debug/mqtt-broker --client target/debug/mqtt-client --report /tmp/c38-phase1.json
```

O harness usa somente loopback e diretórios temporários; verifica4096/4097 QoS0/1/2, keepalive1/timeout1, SUBSCRIBE65536/65537 e recusa por limite de assinaturas. Confira o [registroC38](prompts/registros/C38-limites-fase1.md): preparação ou fixtures não equivalem ao aceite Unix.

## Aceite final da fase1 — C39

Ubuntu26/WSL:52 testes, fmt/Clippy/build e integração com broker0.11.0 aprovados.
Instalação de0.5.0, upgrade para0.7.0, rollback, remove/purge e preservação de
arquivos foram testados em container descartável. Ver [registroC39](prompts/registros/C39-release-fase1.md).
Históricos C37/C38 com BLOCKED foram superados nesse ambiente; não são novos
resultados Windows, Ubuntu24, arm64 ou MSRV1.88 exato.

## Docker — distribuição P63/C40

Imagem lepinheiro100/xmqr-client:0.9.0 para Linux/amd64 com export Docker tar.gz/SHA-256 e manifesto na prerelease; também inclui
.debUbuntu26amd64. [Build/load, configuração e execução](docs/docker.md).
Sem chaves/estado na imagem; usuário10001, mTLS e volumes externos.
