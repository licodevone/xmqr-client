# C147 — múltiplos mapeamentos e diagnóstico estático

2026-10-19, Ubuntu26.04/WSL. Entrada `05472a0`, prompt C147 v1.1.
Client0.42.1/bridge0.4.0 continuam sendo as versões publicadas.

PASS local: TOML v2 com até oito rotas, sufixos de identidade estáveis, estado
QoS2 privado por rota, --check nas duas ordens sem conexão/criação de diretórios,
validação integral antes de iniciar workers e encerramento por falha fatal.
Origem com QoS menor que a assinatura é aceita; journal segue QoS efetivo.
Compatibilidade v1 preservada. Sem dependências novas.

Execução: CARGO_TARGET_DIR=/var/tmp/xmqr-client-p92-native-e688c75,
XMQR_CLIENT_BIN e XMQR_BRIDGE_BIN apontando debug nesse diretório.
`python3 -B scripts/verify_bridge_mappings.py --broker-root ../xmqr --report
dist/c147-local-2.json`: 21 PASS/0 FAIL. Inclui dois brokers reais mTLS, cinco
combinações de QoS, reinício, corrupção preservada, negativos e verificação
de portas sem conexões durante --check.
[Relatório](../../docs/conformance/bridge-mappings-c147.json).

FAIL preservado: primeira execução teve 13 PASS e uma falha de ACL da assinatura
observadora. A fixture omitia tópicos exatos, corrigida após revisão do prompt;
nenhuma ACL real foi alterada.
[Primeiro relatório](../../docs/conformance/bridge-mappings-c147-first-fail.json).

PASS: fmt --check, cargo test --locked --offline, Clippy --all-targets -D warnings,
build --bins e inventário. Regressões verify_bridge_brokers.py
--mqtt5-retain --qos2 (incluindo dois checkpoints de crash) e --qos1 PASS.
CI da fonte exata será vinculada após push; não é PASS por antecipação.

CI confirmada após publicação: [39271798362](https://github.com/licodevone/xmqr-client/actions/runs/39271798362),
commit8d34c240bbfb3b4d9b0f6f65d838ebd927bb97bc, cinco jobs concluídos com
sucesso, incluindo novos21cenários no job bridge-integration, regressões
XMQR/Mosquitto, imagem e instalação/rollback descartáveis. Não houve release
nova nem movimentação de tags.

Limites: destino continua MQTT3.1.1, sem propagação de metadados nem percurso
externo. Journal legado não contém payload/outbox do destino. C148 preparado
para esses requisitos. Sem bump, pacote, imagem ou tag nesta fatia.
T02/fase8 permanece aberto.
