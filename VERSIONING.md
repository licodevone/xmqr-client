# Versionamento

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
