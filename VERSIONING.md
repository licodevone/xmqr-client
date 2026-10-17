# Versionamento

O candidato 0.27.0 reúne C97–C99: solicitação do Client Identifier MQTT 5
ao broker, reutilização manual de ID longo e sessão durável com expiração.
O broker 0.30.0/P166 é o par. Não há persistência do estado QoS da CLI;
MQTT 3.1.1 mantém o limite de 23 caracteres. Tag e prerelease serão publicadas
após CI do commit exato, pacote, imagem e hashes conferidos. Para voltar ao
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
