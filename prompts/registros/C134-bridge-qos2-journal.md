# C134 — bridge QoS 2 com journal da origem

Prompt: [C134 v1.9](../clients/134-bridge-qos2-journal.md), criado e versionado antes da implementação.
Base: client 0.39.0 em `19cc0e9`; branch `feature/c134-bridge-qos2`.
Estado: fatia C134 aceita na fonte; distribuição em nova versão e fase 8/T02 global continuam pendentes.

## Entrega

- `route.qos=2` usa assinatura e publicação MQTT 3.1.1 QoS 2. A bridge só confirma PUBREC na origem depois do PUBCOMP do destino e da gravação do journal.
- Journal privado com lock exclusivo, limite de 32 identificadores e 64 KiB, hash do payload, identidade da rota, escrita atômica e flush. Retomada exige Session Present=1; sessão perdida, arquivo divergente ou corrompido são recusados sem apagar pendências.
- O fork `rumqttc` restaura os identificadores de PUBLISH QoS 2 já aceitos para concluir PUBREL/PUBCOMP após reinício. A unit opcional cria o diretório privado da instância.
- O destino usa sessão persistente somente no QoS 2 para retomar a troca PUBREC/PUBREL/PUBCOMP após reconexão; QoS 0/1 não foram alterados. A release 0.39.0 não inclui C134.

## Evidência local

| Verificação | Resultado |
| --- | --- |
| `cargo fmt --all -- --check`, `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo build --locked` no WSL Ubuntu 26 | PASS |
| Unitário específico da API de restauração no fork vendorizado | PASS |
| Dois XMQR TLS/mTLS QoS 2: entrega, retained, reconexão, ACL, CA/SAN/senha, dois checkpoints de crash, restart determinístico do destino com PUBLISH em voo e perda de Session Present com journal preservado | PASS local; CI da revisão corrigida pendente |
| XMQR→Mosquitto e Mosquitto→XMQR QoS 2 | PASS |
| Regressões QoS 0/1 e topologia de três brokers | PASS |
| `systemd-analyze verify` diretamente na fonte montada `/mnt/d` | BLOCKED local: `/usr/bin/mqtt-bridge` não está instalado no WSL. O job de pacote verificou a unit após instalação descartável (PASS). |
| [CI de `cb98852`](https://github.com/licodevone/xmqr-client/actions/runs/39157647777) | FAIL no job `bridge-integration`: subscriber não recebeu QoS 2 após restart do destino; Rust, imagem e pacote PASS. |
| [CI de `ed9b4d5`](https://github.com/licodevone/xmqr-client/actions/runs/39158474523) | FAIL no oráculo de crash: journal continha outra obrigação retained além de `/crash`; Rust e pacote PASS, imagem cancelada após a falha. |
| [CI de `7f697958`](https://github.com/licodevone/xmqr-client/actions/runs/39158793553) | PASS nos quatro jobs: Rust, bridge-integration (QoS 0/1/2, XMQR e Mosquitto), ubuntu-package e bridge-image. |

## Falhas observadas e correção

- Clippy apontou estilo/complexidade na primeira versão e depois `too_many_lines` e `ref_option`; foram corrigidos sem suprimir lint. A suíte completa foi repetida.
- A primeira fixture de crash pausou uma mensagem retained antes do tópico de teste; o gancho foi limitado ao tópico específico.
- O novo negativo de perda de sessão tentou reutilizar a sessão com um PUBLISH QoS 2 negado por ACL. O destino fechava a conexão, logo não havia PUBCOMP e a pendência legítima bloqueava a instância serial. O teste foi isolado com IDs e journal próprios; a hipótese intermediária sobre Clean Session do destino foi revertida.
- A CI revelou janela de reinício do destino. O fork preserva PUBREL pendente na reconexão; o destino QoS 2 agora mantém sessão persistente. A fixture nova interrompe o destino depois do PUBLISH, reinicia-o e confere journal vazio e replay retained. A primeira versão dessa fixture exigia retained após entrega viva; o oráculo foi corrigido publicando antes da assinatura.
- A CI seguinte mostrou que uma entrada retained anterior pode coexistir no journal com a entrada `/crash`. O oráculo agora exige a entrada específica e, após retomada, journal inteiramente vazio; não exige exclusividade durante a queda.

## Limites e próximos gates

QoS 2 vale em cada perna MQTT; uma queda entre PUBCOMP no destino e a gravação local pode duplicar encaminhamento. Não há transação distribuída nem garantia de processamento do assinante. Publicações negadas na ACL do destino permanecem pendentes na origem e exigem intervenção de operador. A fase 8/T02 continua aberta para retained vivo, loops externos e distribuição. Não criar tag/release para C134 antes de aceite e pacote próprios.
