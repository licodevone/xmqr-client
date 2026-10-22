# C152 - Assinatura ociosa persistente no Windows

Prompt: [C152](../clients/152-windows-service-idle-subscription.md), v1.6.
Base44e13a4b58fb006e71c0fee46352db035c02db02, versao0.43.0 preservada; sem alteracoes anteriores no cliente,
alem do prompt preparado pelo agente principal. Pedido associado a P247:
servicos inicialmente Running, mas conexao do subscriber encerrou depois90s.

## Alteracao

session.rs representa prazo de aplicacao como Option<Instant>. Depois de
SUBACK, a assinatura em Windows SCM ativo OU --service-config com sink privado
ativo usa None; CLI normal mantem Some(now+90s). Retry aceita ausencia de prazo
somente em conexao estabelecida; nova conexao continua com connect_limit e
orcamento finito de tentativas. Espera SUBACK e handshakes QoS2 mantem os prazos
separados existentes. Limite total explicito e cancelamento externo permanecem
no mesmo select do runtime. Sem alteracao MQTT5, TLS/SAN, ACK/sink ou configuracao
de usuarios. Nenhum servico/instalacao/ACL do host foi alterado.

tokio test-util foi acrescentado somente como dev-dependency nos manifests
normal e windows-service; os locks nao mudaram. Fixtures usam peer TCP efemero
127.0.0.1 e tempo virtual, sem certificados/credenciais reais nem estado instalado.

## Execucao e ocorrencias

- Primeiro release nativo: PASS,17.19s, com cache curto
  C:\Users\licod\AppData\Local\Temp\xp242-cargo e target/windows-msi existente.
- Primeira compilacao dos testes: FAIL E0716 nos arrays temporarios do fixture;
  prompt revisado v1.2 antes de bindings locais; runtime nao mudou por isso.
- Segunda rodada: compilacao PASS; caso CLI retornou timeout mas a mensagem
  especifica esperada era fragil; fixture total/retry ficou bloqueado. Prompt
  revisado v1.3 antes de separar casos e avancar alem do prazo absoluto.
  Processo de teste proprio33775 interrompido por Ctrl+C de write_stdin.
- Consulta de processos nao produziu inventario: transporte exec-server
  desconectado antes de criar shell; depois restaurado. taskkill pelo nome
  exato do teste foi negado com erro de credenciais; nenhum servico foi alvo.
- Rodada corrigida session::tests:11/11 PASS em0.16s. Cobre180s virtuais de
  servico idle e Stop/DISCONNECT, CLI90s apesar de PINGRESP, SUBACK/QoS2 limitados,
  total explicito, reconexao finita e cancelamento existente.

Primeira suite completa:97PASS/3FAIL por OsError5 em fixtures privadas:
credentials::tests::windows_private_password_bounds_and_newline (linha104),
service_config::tests::config_rejects_unknown_fields_oversize_and_non_client_commands
(linha119) e service_config::tests::subscriber_requires_jsonl_sink_and_bounded_absolute_configuration
(linha185). Falhas nas escritas dos fixtures depois da preparacao de ACL.
Prompt v1.4 antes de trocar somente TMP/TEMP por pasta nova em Temp do usuario:
o mesmo binario de testes passou100/100 em0.36s. Nenhuma validacao foi enfraquecida;
git diff --exit-code confirma credentials.rs/service_config.rs/windows_private.rs
e os dois Cargo.lock inalterados.

Clippy inicialmente encontrou somente duas conversoes usize->u8 no frame QoS2
de teste. Prompt v1.5 antes de usar try_from. Clippy final all-targets com
-Dwarnings:PASS em1.71s, CARGO_INCREMENTAL=0. fmt --all --check:PASS.
Release nativo final locked/offline:PASS em6.08s, cache xp242-cargo e
target/windows-msi existente; probe --windows-service-capabilities retorna
schema1/componentclient/service_entrypointtrue. Sem nova compilacao aws-lc-sys
no release. Artefato bruto, antes de recursos de marca do agente principal:
`target/windows-msi/x86_64-pc-windows-msvc/release/mqtt-client.exe`.
SHA256: `419926d6d49e072ea0733fd264dd9cd8d66eddb5f30e8a7feaaf224adf4ff0aa`.

Rodada final apos ajuste checked-conversion:100/100 PASS em0.45s,
compilacao7.59s, com TMP/TEMP privado novo e CARGO_INCREMENTAL=0. Executavel
bruto final mede5.876.736bytes e mantem SHA256 acima. git diff --check:PASS.
Gates aqui usam manifest packaging/windows-service;
nao e alegada nova execucao Unix ou do manifesto CLI normal/integracoes externas.

Atualizacao do binario instalado fica com o operador; agente principal prepara
backup/verificacao e aplicacao administrativa. Sem commit/push/tag/release/MSI.

## Complemento de recurso nativo v1.6

O agente principal informou que uma copia alterada depois da compilacao passou
auditoria PE, mas sua execucao foi recusada e o arquivo desapareceu. A causa
externa nao foi comprovada. Nenhuma protecao do host foi alterada nesta etapa.

packaging/windows-service/build.rs agora compila um recurso `1 ICON` com o
rc.exe do Windows SDK e passa o .res somente ao link de mqtt-client. O asset
assets/client.ico e copia byte a byte do icone Client mantido pelo Broker:
SHA256 `97ec36aca17db005c830808e21afee80e735d37adab77d391a6130375a6b43c0`.
Sem dependencia nova, RUSTFLAGS global ou alteracao PE posterior ao link.
RC explicito, PATH e SDK configurado precedem o SDK10.0.26100.0 instalado.
Ausencia/falha do compilador ou target Windows nao MSVC falha claramente.

Validacao nativa novamente, cache xp242-cargo/target windows-msi e
CARGO_INCREMENTAL=0: fmt PASS; Clippy all-targets -Dwarnings PASS3.26s;
suite100/100 PASS0.40s (compilacao10.19s), TMP/TEMP privado exclusivo;
release locked/offline PASS7.50s; git diff --check PASS. Nenhuma falha nesta
rodada. Executavel resultante executou capabilities e version normalmente:
schema1/componentclient/service_entrypointtrue, mqtt-client0.43.0.

Artefato final nativo com recurso, sem pos-edicao:
`target/windows-msi/x86_64-pc-windows-msvc/release/mqtt-client.exe`,
5.912.064bytes, SHA256
`e9228b317a9618ee7502b7aa007d822f48ad07b109e8ac85fceceb72e014bf51`.
Agente principal confirmou original e copia executando capabilities/version;
auditoria readonly PASS para9imagens/10recursos, unchanged=true e SHA acima.
Empacotamento fica com o agente principal. Nenhuma mutacao apos link,
instalacao/SCM/credencial real foi realizada nesta rodada.
