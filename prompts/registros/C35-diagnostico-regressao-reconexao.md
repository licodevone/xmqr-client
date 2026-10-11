# C35 rev1 — execução 2026-10-11

Diagnósticos estáticos para recusas de autenticação/autorização/protocolo/ClientID
e falhas TLS/transporte, sem formatar conteúdo potencialmente sensível. Teste
confirma classificação e supressão de erro interno com conteúdo secreto fictício.
Reconexão/segurança/limites não modificados.

C27 corrigido para QoS1 conforme OASIS3.1.1 §4.3.2/4.4: exige before e after,
aceita retransmissões dessas mensagens sob deadline, verifica tópico e encerra
por SIGINT/exit0. Antes exigia exatamente duas entregas, podendo falhar quando
before era retransmitida. Não excluir investigação de outros possíveis defeitos.

Ubuntu26.04/WSL2: fmt PASS; test --locked PASS (29 unit +16 wire =45); clippy
--locked --all-targets -- -D warnings PASS; build --locked PASS. C27 quatro
cenários em três rodadas PASS; integração shutdown3/LastWill9 PASS com broker
independente. Licenças PASS com Python -X utf8 (sem opção, cp1252 falhou).
Links/sintaxe bash e diff --check PASS.

README descreve assinatura da telemetria e necessidade de ACL/certificado/ID
distinto. Telemetria ESP32 real NOT_RUN nesta execução; não usar credenciais
reais automaticamente nem registrar payloads. Sem tag/release ou versão nova.
Commit/push autorizados na conversa.
