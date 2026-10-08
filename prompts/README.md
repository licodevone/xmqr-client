# Prompts do xmqr-client

Estado auditado em 2026-10-08: pacote xmqr-client, binário mqtt-client, manifest
0.3.0 candidato independente (C28), Rust 2024/MSRV 1.88, MIT, publish=false. Nenhuma release
decorre deste índice. C26 foi escrito antes do código; revisões precederam seus ajustes.

## Ordem de uso atual

1. Ler [AGENTS](../AGENTS.md), [README](../README.md), [ORIGIN](../ORIGIN.json),
   [skill local](../.agents/skills/xmqr-client-development/SKILL.md) e [matriz](COBERTURA.md).
2. Para manutenção do projeto existente, ler C24 e comportamento real dos fontes;
   C25 explica a separação, C26 registra o primeiro incremento independente;
   [C27](clients/27-bootstrap-independente.md) especifica o bootstrap atual.
3. Antes de outra melhoria, criar um novo prompt a partir de [TEMPLATE](TEMPLATE.md),
   com ID/versão/estado/escopo/arquivos/aceite/testes, sem executar propostas históricas.
4. Validar gates e gravar registro real PASS/FAIL/BLOCKED; publicar somente quando
   solicitado, com a versão própria já alinhada em0.3.0. Não reescrever tags existentes.

| ID | Arquivo | Papel e alinhamento |
| --- | --- | --- |
| C19 | [Escopo](clients/19-client-pubsub-scope.md) | Histórico: deixa parâmetros indefinidos; não especifica o cliente atual inteiro. |
| C20 | [Implementação](clients/20-client-pubsub-implementation.md) | Histórico QoS0/codec próprio; não reaplicar em lugar de rumqttc. |
| C21 | [Testes](clients/21-client-pubsub-tests.md) | Histórico: Mosquitto/agentes/codec original; não é evidência nem autoriza instalação. |
| C22 | [QoS e sessões](clients/22-qos-retained-sessions.md) | Requisitos históricos; nova retransmissão não é instrução executável atual. |
| C23 | [Interoperabilidade](clients/23-interoperabilidade-v020.md) | Histórico broker 0.2.0; gate externo ainda não comprovado. |
| C24 | [Auditoria](clients/24-cli-atual-e-credenciais.md) | Manutenção do código real rumqttc; caminhos src/bin/... pertencem à origem. |
| C25 | [Extração](clients/25-extrair-xmqr-client.md) | Exige fonte/backup da origem; extração histórica; integração isolada posterior no registro C27. |
| C26 | [Reconexão e JSONL](clients/26-reconexao-jsonl.md) | Incremento implementado sobre projeto existente; [registro](registros/C26-reconexao-jsonl.md). |
| C27 | [Bootstrap independente](clients/27-bootstrap-independente.md) | Contrato consolidado até C26; reconstrução nova em pasta inicialmente vazia; [registro](registros/C27-bootstrap.md). |
| C28 | [Payload binário C28](clients/28-payload-binario-planejado.md) | Rev.2.0 executada: arquivo/stdin bruto e por linha; [registro](registros/C28-payload-binario.md). |

## Posso criar o cliente do zero usando estes prompts?

Agora há um contrato independente atual: **C27**, cobrindo CLI/defaults,
dependências, TLS/SAN/mTLS/credenciais, QoS/Will/sessões, reconexão e JSONL até C26.
Foi redigida uma implementação nova em [validation/c27-bootstrap](../validation/c27-bootstrap/README.md),
num diretório que não existia; não foram copiados fontes src/ de produção ou broker.
Manifest/lock/licença e fixtures C26 foram reutilizados de forma explícita,
com hashes e mudanças aprovadas. O mesmo autor conhecia C26; não foi ensaio cego.

Isso fornece um caminho de reconstrução pelo contrato **com os insumos declarados**,
não uma promessa de equivalência bit a bit, todos os comportamentos não testados
ou dependências disponíveis em outra máquina. O Cargo.lock é dado necessário
para resolução idêntica; versões de manifesto sozinhas não fixam as transitivas.
Veja as plataformas, cenários PASS e lacunas no registro C27 antes de assumir aceite.
MSRV1.88 exato e interoperabilidade com Mosquitto continuam não comprovados.

C25 ainda pressupõe fontes/backup da origem e serve para entender a extração.
C19–C23 preservam links históricos (contrato-base, prompts/origem, broker/36)
ausentes aqui; C20 descreve codec próprio antigo e não deve substituir rumqttc.
Para um novo ensaio, use C27 em outro diretório vazio, sem sobrescrever a evidência
existente ou executar histórico C20. Não executar broker compartilhado sem coordenação.

C28 rev.2.0 amplia o contrato com arquivo/stdin binário e publicação por linha;
versão candidata0.3.0. Veja o suplemento C28 no contrato C27 e seu registro.
A reconstrução validation/c27-bootstrap permanece congelada no baseline0.2;
seus hashes antigos não demonstram equivalência com produção após C28.
Perfis, múltiplos tópicos e timestamps não foram implementados; não avançar C29
antes do checkpoint de publicação. Tag sugerida **v0.3.0**, ainda não criada aqui.
