# C97 — Client Identifier atribuído aceito parcialmente

O [prompt C97](../clients/97-assigned-client-id-mqtt5.md) foi commitado antes da funcionalidade (`42be0c7`). Fonte exata `8f7e8e867891755b7bc7492ad322f463143260e2`: 73 testes de CLI, 65 de wire, fmt, Clippy, build release, inventário de licenças e 7 contratos de distribuição PASS localmente; [CI PASS](https://github.com/licodevone/xmqr-client/actions/runs/39021858472), incluindo pacote Ubuntu 26.04.

`--server-assigned-client-id` envia ID vazio somente em MQTT 5 com sessão limpa, sem expiração e sem `--client-id` manual. O client exige ID não vazio e válido na propriedade `0x12` do CONNACK, rejeita propriedade não solicitada e duplicata recebida, e mostra o ID em stderr. O ensaio `xmqr/scripts/verify_mqtt5_assigned_client_id.py` com broker P166 retornou `P166_C97_ASSIGNED_ID_PASS`: dois IDs distintos, pub/sub real e saída JSONL preservada. A CLI não promete retomada automática do estado QoS local.

Sem tag/release isolada. Fase 6A global e interop externa ampliada continuam abertas.
