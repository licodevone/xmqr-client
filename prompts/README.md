# Prompts do xmqr-client

Estado auditado em 2026-10-08: pacote xmqr-client, binário mqtt-client, manifest
0.7.0 local herdado, Rust 2024/MSRV 1.88, MIT, publish=false. Nenhuma release
decorre deste índice. C26 foi escrito antes do código; revisões precederam seus ajustes.

## Ordem de uso atual

1. Ler [AGENTS](../AGENTS.md), [README](../README.md), [ORIGIN](../ORIGIN.json),
   [skill local](../.agents/skills/xmqr-client-development/SKILL.md) e [matriz](COBERTURA.md).
2. Para manutenção do projeto existente, ler C24 e comportamento real dos fontes;
   C25 explica a separação, C26 registra o primeiro incremento independente.
3. Antes de outra melhoria, criar um novo prompt a partir de [TEMPLATE](TEMPLATE.md),
   com ID/versão/estado/escopo/arquivos/aceite/testes, sem executar propostas históricas.
4. Validar gates e gravar registro real PASS/FAIL/BLOCKED; publicar somente quando
   solicitado, depois de decidir versão própria. Não reescrever tags existentes.

| ID | Arquivo | Papel e alinhamento |
| --- | --- | --- |
| C19 | [Escopo](clients/19-client-pubsub-scope.md) | Histórico: deixa parâmetros indefinidos; não especifica o cliente atual inteiro. |
| C20 | [Implementação](clients/20-client-pubsub-implementation.md) | Histórico QoS0/codec próprio; não reaplicar em lugar de rumqttc. |
| C21 | [Testes](clients/21-client-pubsub-tests.md) | Histórico: Mosquitto/agentes/codec original; não é evidência nem autoriza instalação. |
| C22 | [QoS e sessões](clients/22-qos-retained-sessions.md) | Requisitos históricos; nova retransmissão não é instrução executável atual. |
| C23 | [Interoperabilidade](clients/23-interoperabilidade-v020.md) | Histórico broker 0.2.0; gate externo ainda não comprovado. |
| C24 | [Auditoria](clients/24-cli-atual-e-credenciais.md) | Manutenção do código real rumqttc; caminhos src/bin/... pertencem à origem. |
| C25 | [Extração](clients/25-extrair-xmqr-client.md) | Exige fonte/backup da origem; separação estrutural registrada, integração pendente. |
| C26 | [Reconexão e JSONL](clients/26-reconexao-jsonl.md) | Incremento implementado sobre projeto existente; [registro](registros/C26-reconexao-jsonl.md). |

## Posso criar o cliente do zero usando estes prompts?

Ainda não há uma receita independente completa nem reconstrução executada.
C25 pressupõe copiar fontes existentes do broker/backup. C19–C23 mencionam
contrato-base.md, prompts/origem, broker/36 e agentes que não estão neste projeto.
docs/original-mqtt-client.md conserva referências da origem. C20 descreve uma
fatia antiga com codec próprio; C24 prevalece para o cliente atual rumqttc.
Esses links históricos são documentação da procedência, não dependências de build.
Build/testes atuais funcionam sem importar ou executar o crate do broker.

Próxima etapa proposta: C27, especificação de bootstrap independente. Deve reunir
manifest/features/lock, CLI completa/defaults/erros, transporte TLS/SAN/credenciais,
contratos MQTT/QoS/Will/sessões/reconexão/JSON, limites/deadlines e fixtures autônomas;
eliminar a necessidade de copiar fontes do broker e resolver os links normativos.
Só depois validar em diretório limpo, sem usar os fontes atuais como entrega pronta,
e registrar gates Windows/Linux e integrações externas. Essa etapa não foi executada
e não constitui uma promessa de reprodutibilidade ou autorização de instalação.
