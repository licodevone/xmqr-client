# Registro C153 — verificação de configuração de pub/sub

- Prompt C153 v1.1. Entrada `a6d36f7`, client 0.43.0/bridge 0.5.0.
- `--check-config pub|sub` analisa as opções normais e verifica materiais
  TLS e arquivo privado de senha quando aplicáveis. Saída JSON redigida.
  README e comparação com `mosquitto_pub`, `mosquitto_sub` e `mosquitto_rr`.
- Windows/Rust 1.99.1: compilação direta com rlibs do cache C151 e
  três testes dirigidos PASS. O primeiro teste com perfil em modo de
  laboratório foi recusado pelo contrato existente. C153 v1.1 usa
  perfil TLS e verifica a recusa do material inexistente.
- Gate Cargo offline BLOCKED: `windows-permissions` ausente do índice
  local. A compilação com rlibs não substitui Cargo locked e CI Linux.
- Sem alteração do protocolo, bridge ou estado. Request/response,
  template e taxa permanecem no backlog.
