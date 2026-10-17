# C101 rev1 — diagnóstico seguro de DISCONNECT MQTT 5 do servidor

Entrada: broker P175 emite DISCONNECT `0x95` após pacote excessivo; a biblioteca MQTT 5 do client transporta Reason String do servidor em seu erro e a CLI imprime o erro bruto. O [MQTT 5 OASIS](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html), seção 3.14, trata Reason String como texto diagnóstico não interpretável. Este prompt precede o código.

Objetivo: quando o servidor envia DISCONNECT MQTT 5 após CONNACK, mostrar o nome/código do motivo, sem incluir Reason String ou User Property no stderr. Não mascarar erros locais, transporte ou CONNACK. Preservar código de saída não zero, comportamento pub/sub e MQTT 3.1.1. Não ecoar payload/credenciais.

Aceite: teste unitário com Reason String sentinela, servidor TCP simulado que envia DISCONNECT com motivo `0x95` e texto sensível, asserts de stderr/exit e regressões pub/sub; fmt, Rust, Clippy, build release, licenças/distribuição e CI. Checklist e registro parciais; sem tag isolada.
