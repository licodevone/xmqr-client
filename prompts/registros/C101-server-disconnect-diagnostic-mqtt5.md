# C101 — diagnóstico seguro de DISCONNECT MQTT 5 aceito parcialmente

O [prompt C101](../client/101-server-disconnect-diagnostic-mqtt5.md) foi commitado antes da funcionalidade (`589a98a`). Fonte exata `cec59b2c4b05f607118182fb3f13eb621dad49a0`: 77 testes unitários e 66 testes de rede PASS, fmt, Clippy, build release, inventário de licenças e 7 contratos de distribuição PASS localmente; [CI PASS](https://github.com/licodevone/xmqr-client/actions/runs/39026544541) nos jobs Rust e pacote Ubuntu 26.04.

Ao receber DISCONNECT MQTT 5 do servidor após CONNACK, a CLI mostra nome e código hexadecimal do motivo, sem reproduzir Reason String arbitrária. O teste TCP simulado enviou `0x95` com `secret-fixture-content`: stderr continha `PacketTooLarge (0x95)`, omitia o texto e o processo saiu com erro. O teste unitário cobre também texto com quebra de linha e confirma que erro local mantém seu diagnóstico. Pub/sub e MQTT 3.1.1 passaram na regressão.

Sem alteração do protocolo no wire, migração ou tag/release isolada. Não há reconexão automática por esse motivo; aceites globais da fase 6 continuam abertos.
