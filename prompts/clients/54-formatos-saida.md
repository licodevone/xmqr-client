# C54 rev1 — formatos de saída binária (C06)

2026-10-15. Entrada: client0.12.0 publicado, C52/C53 implementados e validados
localmente no Ubuntu26. `sub` oferece `text` e `jsonl`; JSONL preserva bytes em
array, mas não há fluxo binário direto nem representação hexadecimal compacta.

Decisão: adicionar `--output hex` e `--output raw` somente em `sub`. `hex` escreve
uma linha por PUBLISH com tópico e payload em hexadecimal, QoS e retain, sem
escape ambíguo. `raw` escreve os bytes do payload sem separador e exige
`--count 1`; assim não concatena mensagens sem framing. Status vai para stderr
nos formatos de máquina. Sem timestamp sintético: o client não conhece horário
de publicação do broker e não deve apresentá-lo como tal.

Escopo: parser, módulo output, sessão, testes, README/CHANGELOG/matriz/registro.
Preservar `text`/`jsonl` byte a byte; validação de modo/contagem antes da rede.
Erros de stdout não podem ser tratados como sucesso. Filtros retained/exclusão
e `--count` são aplicados antes da escrita. Não mudar wire, ACL ou payload.

Aceite: fmt/test/Clippy/build/inventário; bytes nulos/UTF-8 inválido, tópico
Unicode, saída sem newline para raw, erro de modo/contagem, ACKs QoS1/2 e
regressão de JSONL. Não promover fase 5 completa apenas por C54.
