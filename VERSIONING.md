# Versionamento

O 0.30.0 candidato reúne C107/C108: outbox QoS 2 de uma publicação de saída
por diretório e manifesto de uma assinatura MQTT 5 contínua no Linux/WSL.
Ambos exigem Client Identifier fixo e Session Present para retomada de estado
ativo. PUBLISH/PUBREL e SUBACK têm transições gravadas antes de anunciar
sucesso; mensagens recebidas não são persistidas pela CLI. Diretórios 0700
e arquivos 0600 contêm payload ou declaração em claro, sem senhas. A tag e a
prerelease dependem do CI da fonte exata, pacote, upgrade/rollback descartável,
imagem por digest e hashes remotos. MQTT 3.1.1 permanece padrão; fases
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
