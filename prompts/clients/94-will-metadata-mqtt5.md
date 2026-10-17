# C94 rev1 — metadados do Will MQTT 5 no client

Entrada: C70 envia Will Delay e o `rumqttc` fixado já suporta os campos de `LastWillProperties`. P163 prepara o broker para receber os metadados. Este prompt precede o código.

Objetivo: oferecer opções explícitas do Will somente em MQTT 5 quando há Will: `--will-payload-format 0|1`, `--will-content-type`, `--will-response-topic`, `--will-correlation-data-hex` e `--will-user-property chave=valor` repetível. Aplicar limites existentes de PUBLISH e preservar ordem dos pares. Conservar comportamento legado e permitir combinação com Will Delay. Não aceitar Message Expiry do Will nesta parcela nem reinterpretar as flags de PUBLISH como metadados do Will. Rejeitar opção sem Will, MQTT 3.1.1, binário inválido, duplicata escalar e valores fora de quota antes da rede.

Aceite: CLI e wire CONNECT com propriedades/ordem corretas, negativos, P163 em TCP real com Will e assinante recebendo metadados, regressão C70/3.1.1; fmt, Rust, Clippy, build release, licenças, contratos de distribuição e CI do commit exato. Registrar parcela 6A/6B sem tag isolada nem afirmar Will Properties completas.

Fonte: [MQTT 5 §3.1.3.2](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html).
