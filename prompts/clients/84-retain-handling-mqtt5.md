# C84 — opção Retain Handling do assinante MQTT 5

Entrada: C83 aceito e broker P147 planejado. Este prompt precede o código C84.

Objetivo: expor `sub --protocol-version 5 --retain-handling 0|1|2`, aplicando a opção a cada filtro do SUBSCRIBE. Zero preserva o padrão; um pede retained só quando a assinatura ainda não existe; dois não pede retained no ato da assinatura. Preservar filtros `--ignore-retained`/`--retained-only` como filtros locais de saída independentes, sem fingir que alteram a opção wire. Rejeitar modo pub e MQTT 3.1.1 antes de abrir rede. Sem alteração de perfil TOML v1 ou do estado local.

Aceite: CLI e wire para 0/1/2 e negativos; integração TCP real com broker P147 e Mosquitto 2.0.22, incluindo sessão persistente/restart; regressão sub QoS0/1/2 e filtros retained; fmt/test/Clippy/build/licenças/CI. Checklist após aceite. Sem release isolada ou ensaio ESP32.

Fonte normativa: OASIS MQTT 5 §3.8.3.1, https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html.
