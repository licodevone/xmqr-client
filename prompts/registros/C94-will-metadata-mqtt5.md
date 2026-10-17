# C94 — aceite parcial de metadados do Will MQTT 5

Prompt `f12a5b2` precedeu a funcionalidade `f330dd44d5a7282a1388eeda2a93c42b0ea18a09`. CI desse commit: [PASS](https://github.com/licodevone/xmqr-client/actions/runs/39019171771), incluindo Rust e pacote Ubuntu 26.04. Fmt, 68 testes CLI e 65 de wire, Clippy, build release, inventário de licenças e 7 contratos de distribuição passaram localmente.

Flags Will específicas em MQTT 5 validam protocolo, Will completo, formatos, tamanho e quantidade antes da rede. O CONNECT preserva a ordem de User Property e combina os metadados com Will Delay. Ensaio cruzado real `xmqr/scripts/verify_mqtt5_will_metadata.py` PASS com broker P163: entrega imediata, atraso/restart, retained e offline. Os argumentos de PUBLISH permanecem separados dos de Will; MQTT 3.1.1 não aceita essas flags.

Message Expiry do Will segue em C95/P164. A release 0.25.0 não contém C94; não há tag isolada. Fases 6A/6B globais continuam abertas.
