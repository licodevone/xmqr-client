# C57 rev1 — publicação MQTT 5 inicial

Data: 2026-10-16. Entrada: `main` em `f447488`, client 0.13.0. Solicitação: prosseguir na fase 6A com prompt antes do código.

## Resultado observável

Adicionar escolha explícita `--protocol-version 3.1.1|5` (padrão 3.1.1). Em 5, permitir `pub` QoS 0/1 com Clean Start e sem Will, usando a biblioteca MQTT 5 já presente, transporte/credenciais TLS existentes, limites e timeouts. Interpretar CONNACK de recusa como erro claro, aguardar confirmação QoS 1 e encerrar com DISCONNECT. Validar opções incompatíveis antes da rede. `sub` MQTT 5, QoS 2 e propriedades avançadas ficam pendentes.

## Invariantes e aceite

Não alterar o caminho 3.1.1. Não repetir publicação de resultado incerto; preservar senha fora de argv, CA/SAN, perfis e quotas. Testar parser, sucesso QoS0/1 com broker XMQR P115, recusa e regressão 3.1.1. Executar fmt/testes/Clippy/build, registrar em `prompts/registros/C57-publicacao-mqtt5-inicial.md`. Sem tag/release isolada até marco validado.

Norma: <https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html>; API local rumqttc 0.25.1 v5.

Rev2: o primeiro recorte aceita uma publicação por processo, recusando `--repeat`, `--line-mode` e reconexão na versão 5; perfil/ambiente aceitam a escolha de versão com a precedência já estabelecida. Teste de wire cobre CONNACK com Maximum QoS=1 e Maximum Packet Size=65536 emitido por P115. Sinais encerram a operação sem replay.
