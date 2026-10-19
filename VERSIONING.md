# Versionamento

Candidata 0.42.0/bridge 0.4.0: C139 e cinco correções obrigatórias da auditoria
P232 (C140–C142). Minor experimental porque inclui a nova origem MQTT5/QoS2,
não apenas patches. [Notas e migração](docs/releases/0.42.0.md) e
[aceite C143](prompts/registros/C143-five-audit-fixes.md). Tag/pacote/imagem
somente após gates; não alterar v0.41.0 ou latest. Broker 0.43.0 e admin
independente 0.11.0 não mudam. Formatos duráveis não são migrados por essas
cinco correções. Reverter para 0.41.0 reintroduz os cinco defeitos; a bridge
antiga não aceita origem MQTT5/QoS2 com o journal C139.

A prerelease 0.41.0 agrupa C136/C137: origem MQTT 5/QoS 0/1 da bridge
com Retain As Published; `mqtt-bridge 0.3.0` acompanha `mqtt-client 0.41.0`.
O destino ainda usa MQTT 3.1.1; QoS 2 MQTT 5, propriedades propagadas,
ciclos externos e transação entre brokers ficam fora do recorte. O contrato
finito da fase 6 foi aceito no broker 0.43.0; fase 8 permanece aberta.
[Release](https://github.com/licodevone/xmqr-client/releases/tag/v0.41.0),
[nota](docs/releases/0.41.0.md) e [registro C138](prompts/registros/C138-release-0.41.0.md).

A prerelease 0.40.0 acrescenta C134 à bridge experimental: QoS 2 nas duas
pontas, journal da origem, retomada condicional e unit com StateDirectory
privado. `mqtt-bridge 0.2.0` acompanha `mqtt-client 0.40.0`; 0.39.0 não
contém QoS 2 na bridge. [Release](https://github.com/licodevone/xmqr-client/releases/tag/v0.40.0),
[nota](docs/releases/0.40.0.md), [C134](prompts/registros/C134-bridge-qos2-journal.md)
e [C135](prompts/registros/C135-release-0.40.0.md). Fonte exata/CI, pacote,
imagem por digest e seis anexos remotos verificados; na publicação 0.40.0,
as fases 6 e 8 ainda estavam abertas.

A prerelease 0.39.0 integra a bridge experimental C125–C128 ao pacote e imagem
do client 0.38.0, preservando o journal inbound C129. O `.deb` e a imagem
passam a conter `mqtt-client` e `mqtt-bridge 0.1.0`; o serviço é opt-in.
[Contrato e limites](docs/bridge-one-way.md),
[nota](docs/releases/0.39.0.md) e
[registro C133](prompts/registros/C133-bridge-package-039.md).
Tag da fonte exata, CI, pacote, imagem por digest e seis anexos remotos
verificados. Fases 6 e 8 globais abertas.

A prerelease 0.38.0 distribui somente `mqtt-client`/C129. Documento inbound
v2 aceita draft v1 válido sem escrita na leitura; os demais journals e
manifestos mantêm versão 1. Client 0.36.0 não conhece inbound C129: downgrade
para essa versão não autoriza retomar estado pendente com o binário antigo.
[Nota](docs/releases/0.38.0.md) e
[registro](prompts/registros/C131-release-0.38.0.md).

A prerelease 0.36.0 distribui C122/C123: outbox QoS 2 e manifesto de assinatura
MQTT 5 por WSS mTLS, com identidade de URL canônica. Sem migração dos journals
TCP; 0.35.0 lê os arquivos existentes, mas recusa essas opções por URL. Broker
0.42.0/P216 permanece o par distribuído. CI da fonte exata, pacote Ubuntu,
rollback descartável, imagem por digest e seis hashes remotos PASS.
[Release](https://github.com/licodevone/xmqr-client/releases/tag/v0.36.0),
[nota](docs/releases/0.36.0.md) e [registro](prompts/registros/C124-release-0.36.0.md).
Fase 7 global aberta.

A prerelease 0.35.0 distribui C119/C120: outbox QoS 1 MQTT 5 por WSS mTLS
com URL canônica como identidade e ensaio controlado de retomada. Sem migração
do journal TCP; o 0.34.0 preserva dados, mas recusa a opção por URL. Broker
0.42.0/P216 é o par distribuído. CI da fonte exata, pacote Ubuntu, rollback
descartável, imagem por digest e seis hashes remotos PASS.
[Release](https://github.com/licodevone/xmqr-client/releases/tag/v0.35.0),
[nota](docs/releases/0.35.0.md) e [registro](prompts/registros/C121-release-0.35.0.md).
Fase 7 global aberta.

O 0.34.0 publicado inclui C116/C117: perfil WSS mTLS e acesso remoto somente
com `--allow-remote-wss`. Manifestos duráveis não mudam; o 0.33.0 os lê, mas
não contém esses recursos. Broker 0.41.0/P214 é o par distribuído. CI da fonte
exata, pacote Ubuntu, rollback descartável, imagem por digest e seis hashes
remotos PASS. [Release](https://github.com/licodevone/xmqr-client/releases/tag/v0.34.0),
[nota](docs/releases/0.34.0.md) e [registro](prompts/registros/C118-release-0.34.0.md).
Fase 7 global aberta.

A prerelease 0.33.0 distribui C114 WSS mTLS local sem alterar manifestos
duráveis. A versão anterior 0.32.0 não contém C114. CI, pacote Ubuntu 26.04,
upgrade/rollback descartável, imagem numerada por digest, tag e prerelease
com seis hashes remotos passaram. [Release](https://github.com/licodevone/xmqr-client/releases/tag/v0.33.0),
[nota](docs/releases/0.33.0.md) e [registro C115](prompts/registros/C115-release-0.33.0.md).
Fase 7 global aberta.

O 0.32.0 publicado distribui C112: WebSocket de laboratório em loopback para
pub/sub MQTT 3.1.1/5. Sem WSS, perfis nem estado durável sobre WS. O broker
0.39.0 oferece o listener local P210. CI, pacote Ubuntu 26.04,
upgrade/rollback descartável, imagem numerada, tag, prerelease e seis hashes
remotos passaram. [Nota](docs/releases/0.32.0.md) e
[registro](prompts/registros/C113-release-0.32.0.md). Sem mudança do estado
durável; fase 7 global aberta.

O 0.31.0 publicado distribui C110: ACK QoS 1/2 da assinatura persistente
após escrita/flush; falha de stdout não confirma. Sem journal inbound ou
garantia de processamento do consumidor. Client 0.30.0 lê o mesmo manifesto;
broker 0.37.0 e admin independente 0.10.0 permanecem. CI, pacote Ubuntu
26.04, upgrade/rollback, imagem por digest, tag, prerelease e seis hashes
remotos passaram. [Nota](docs/releases/0.31.0.md) e
[registro](prompts/registros/C111-release-0.31.0.md). M03/6B continuam abertos.

O 0.30.0 publicado reúne C107/C108: outbox QoS 2 de uma publicação de saída
por diretório e manifesto de uma assinatura MQTT 5 contínua no Linux/WSL.
Ambos exigem Client Identifier fixo e Session Present para retomada de estado
ativo. PUBLISH/PUBREL e SUBACK têm transições gravadas antes de anunciar
sucesso; mensagens recebidas não são persistidas pela CLI. Diretórios 0700
e arquivos 0600 contêm payload ou declaração em claro, sem senhas. Tag anotada
e prerelease publicadas após CI da fonte exata, pacote, upgrade/rollback
descartável, imagem por digest e hashes remotos. MQTT 3.1.1 permanece padrão; fases
6A–6D e 7–9 seguem abertas. [Nota](docs/releases/0.30.0.md).

O 0.29.0 publicado reúne C104/C105: Session Expiry explícita no DISCONNECT
MQTT 5 e outbox opt-in para uma publicação QoS 1 de saída no Linux/WSL.
O journal contém payload em claro sob permissões privadas; retomada exige
Session Present=1 e admite duplicata QoS 1. QoS 2, assinaturas e demais fluxos
locais não são restaurados. Tag anotada e prerelease publicadas após CI, pacote,
upgrade/rollback descartável, imagem por digest e hashes remotos conferidos.
MQTT 3.1.1 permanece padrão; fases 6A–6D e 7–9 seguem abertas.
[Nota](docs/releases/0.29.0.md).

A versão 0.28.0 publicada reúne C101/C102: diagnóstico seguro de DISCONNECT MQTT 5 e
User Property contextual de SUBSCRIBE/UNSUBSCRIBE. O broker parceiro 0.33.0
contém P180/P181. Sem persistência do fluxo QoS local; MQTT 3.1.1 mantém o
contrato anterior. Tag anotada e prerelease publicadas após CI, pacote Ubuntu
26.04, imagem numerada, upgrade/rollback descartável e hashes conferidos. Fases 6A–6D e
7–9 continuam abertas. [Nota](docs/releases/0.28.0.md).

A versão 0.27.0 publicada reúne C97–C99: solicitação do Client Identifier MQTT 5
ao broker, reutilização manual de ID longo e sessão durável com expiração.
O broker 0.30.0/P166 é o par. Não há persistência do estado QoS da CLI;
MQTT 3.1.1 mantém o limite de 23 caracteres. Tag anotada e prerelease foram
publicadas após CI do commit exato, pacote, imagem e hashes conferidos. Para voltar ao
pacote 0.26.0, pare as unidades e preserve perfis e segredos. Fases globais
6A–6D e 7–9 continuam abertas. [Nota](docs/releases/0.27.0.md).

O candidato 0.26.0 reúne C94/C95: propriedades e Message Expiry do Will MQTT 5,
com broker 0.29.0 como par. Tag anotada e prerelease serão publicadas após
CI do commit exato, pacote, imagem e hashes verificados. As opções de Will
não reinterpretam opções PUBLISH; 0.25.0 não contém C94/C95. Para voltar ao
pacote 0.25.0, pare as unidades e preserve perfis e segredos. Fases globais
6A–6D e 7–9 ainda abertas. [Nota](docs/releases/0.26.0.md).

Versões minor distribuem parcelas novas; patches corrigem defeitos. Tags
identificam o código aceito e não são movidas para incorporar registros
de publicação posteriores.
