# C85 — No Local e Retain As Published no assinante MQTT 5

Entrada: C84 Retain Handling aceito; o broker XMQR ainda rejeita os bits No Local e Retain As Published, previstos para P148. Este prompt precede o código C85.

Objetivo: expor `sub --protocol-version 5 --no-local` e `--retain-as-published`, aplicando os bits 2 e 3 a cada filtro enviado. Ambos desativados por padrão. Aceitar combinação com `--retain-handling 0|1|2`; rejeitar uso no publisher e no MQTT 3.1.1 antes de abrir conexão. Não confundir `--retain-as-published` com os filtros locais de saída `--ignore-retained`/`--retained-only`. Não modificar perfis TOML v1 ou persistência do client.

Aceite: parsing/negativos CLI, bytes SUBSCRIBE e integração real contra broker MQTT 5 externo que suporta as opções, incluindo autopublicação suprimida por No Local e RETAIN preservado em entrega viva. Regressões de C84, fmt/test/Clippy/build/licenças e CI. Marcar apenas C85 parcial até P148 do broker; sem release isolada ou ensaio físico.

Fonte normativa: OASIS MQTT 5 §3.8.3.1, https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html.
