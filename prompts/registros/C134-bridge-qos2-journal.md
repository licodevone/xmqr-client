# C134 — bridge QoS 2 com journal da origem

Prompt: [C134 v1.9](../clients/134-bridge-qos2-journal.md), criado e versionado antes da implementação.
Base: client 0.39.0 em `19cc0e9`; branch `feature/c134-bridge-qos2`.
Estado: implementação local validada; aceite e integração dependem da CI da fonte exata.

## Entrega

- `route.qos=2` usa assinatura e publicação MQTT 3.1.1 QoS 2. A bridge só confirma PUBREC na origem depois do PUBCOMP do destino e da gravação do journal.
- Journal privado com lock exclusivo, limite de 32 identificadores e 64 KiB, hash do payload, identidade da rota, escrita atômica e flush. Retomada exige Session Present=1; sessão perdida, arquivo divergente ou corrompido são recusados sem apagar pendências.
- O fork `rumqttc` restaura os identificadores de PUBLISH QoS 2 já aceitos para concluir PUBREL/PUBCOMP após reinício. A unit opcional cria o diretório privado da instância.
- O destino conserva a configuração de sessão anterior; QoS 0/1 não foram alterados. A release 0.39.0 não inclui C134.

## Evidência local

| Verificação | Resultado |
| --- | --- |
| `cargo fmt --all -- --check`, `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo build --locked` no WSL Ubuntu 26 | PASS |
| Unitário específico da API de restauração no fork vendorizado | PASS |
| Dois XMQR TLS/mTLS QoS 2: entrega, retained, reconexão, ACL, CA/SAN/senha, dois checkpoints de crash e perda de Session Present com journal preservado | PASS |
| XMQR→Mosquitto e Mosquitto→XMQR QoS 2 | PASS |
| Regressões QoS 0/1 e topologia de três brokers | PASS |
| `systemd-analyze verify` diretamente na fonte montada `/mnt/d` | BLOCKED: `/usr/bin/mqtt-bridge` não está instalado no WSL; unit será verificada no job de pacote após instalação descartável. |
| CI da revisão exata | PENDING |

## Falhas observadas e correção

- Clippy apontou estilo/complexidade na primeira versão e depois `too_many_lines` e `ref_option`; foram corrigidos sem suprimir lint. A suíte completa foi repetida.
- A primeira fixture de crash pausou uma mensagem retained antes do tópico de teste; o gancho foi limitado ao tópico específico.
- O novo negativo de perda de sessão tentou reutilizar a sessão com um PUBLISH QoS 2 negado por ACL. O destino fechava a conexão, logo não havia PUBCOMP e a pendência legítima bloqueava a instância serial. O teste foi isolado com IDs e journal próprios; a hipótese intermediária sobre Clean Session do destino foi revertida.

## Limites e próximos gates

QoS 2 vale em cada perna MQTT; uma queda entre PUBCOMP no destino e a gravação local pode duplicar encaminhamento. Não há transação distribuída nem garantia de processamento do assinante. Publicações negadas na ACL do destino permanecem pendentes na origem e exigem intervenção de operador. A fase 8/T02 continua aberta para retained vivo, loops externos e distribuição. Não criar tag/release para C134 antes de aceite e pacote próprios.
