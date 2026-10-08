# C26 - Reconexao limitada e JSON Lines, versao 1.0, 2026-10-08

Autorizacao: retomar implementacoes do client, primeiro marco delegado; sem broker.
Estado: HEAD 26c1019, working tree limpo, origin licodevone/xmqr-client;
manifest 0.7.0, publish=false, tag local v0.1.0 preservada. Sem release.
Skill: .agents/skills/xmqr-client-development/SKILL.md; AGENTS/README/ORIGIN lidos.

Objetivo: preservar texto por padrao; adicionar --output text|jsonl e
--reconnect-attempts 0..10 (padrao 0), espera exponencial 100ms..5s.
Tentativas sao orçamento total do processo, sem reset ao conectar, para impedir
loops infinitos com conexoes que caem imediatamente. Ctrl+C interrompe espera/poll.
Somente erros transitorios de transporte podem retentar; TLS/auth/protocolo falham.
Mesmo AsyncClient/EventLoop e Client ID durante recuperacao. rumqttc 0.25.1
reconecta ao continuar poll, limpa estado em erro e so preserva pending quando
CONNACK session_present=true; nao criar segundo mecanismo nem reenviar PUBLISH.
Publicacao em andamento: falhar com resultado desconhecido apos perda da conexao,
sem repetir publicacao; retries permitidos apenas antes de publica-la.
Assinatura recuperada exige SUBSCRIBE quando sessao ausente; sessao presente
preserva assinatura, sem duplicar pending. Limite --count inclui mensagens antes
SUBACK e reconexao. Logs operacionais stderr em JSON; stdout somente dados JSONL.
JSON payload como array de bytes, sem perda binaria; escaping JSON de controles,
sem credenciais/metadados secretos. ACK e apenas confirmacao de protocolo.

Escopo: src/cli.rs, src/session.rs, novo src/output.rs, src/main.rs; README e
registro C26; testes unitarios e fixtures wire loopback sem broker externo.
Sem dependencias novas, sem alterar Cargo/versao, TLS, limites 1024/4096,
credenciais/0600 Linux, Will, QoS ou laboratorios. Sem instalar/commit/push/tag.

Aceite/testes planejados antes do codigo: CLI default/limites/repeticao/invalida;
JSON vazio, binario, Unicode, controles, QoS/retain e sucesso pub;
fixture MQTT em porta loopback efemera: reconexao sessao presente/ausente,
limite finito em EOF/recusa, count antes SUBACK, pub sem replay apos desconexao,
QoS0/1/2 ACK corretos, texto preservado. Deadlines e numero de conexoes limitados.
fmt --check, test --locked, clippy --locked --all-targets -D warnings, build;
inventario licencas. WSL/integração broker real separados BLOCKED quando indisponiveis.
Registro real prompts/registros/C26-reconexao-jsonl.md com PASS/FAIL/BLOCKED.

## Revisao 1.1 - alinhamento de prompts, 2026-10-08

Pedido adicional: avaliar se prompts atuais permitem reconstruir xmqr-client do zero.
Antes de editar documentacao: ampliar escopo para prompts/README.md (indice),
prompts/COBERTURA.md (matriz) e avisos locais C19-C24 sem apagar historico.
Nao executar bootstrap paralelo. Declarar referencias ausentes, dependencia
historica da origem e gates nao comprovados. C25 e extracao, C26 incremento;
nao prometer reprodutibilidade. Propor C27 independente como proximo trabalho.
Documentar pacote Rust2024/MSRV1.88 e dependencias exatas do manifest/lock atual,
TLS/SAN, credenciais/0600, limites1024/4096, QoS/Will/sessao e novo C26.
Aceite adicional: links novos locais validos; matriz distingue codigo, testes
Windows, Linux/TLS/Will bloqueados e bootstrap inexistente. Manter0.7.0 local.

## Revisao 1.2 - limite real rumqttc QoS2, antes do ajuste

Inspecao State::clean apaga incoming_pub; handle_incoming_pubrel rejeita pkid
que nao esteja nesse estado. Logo recuperacao de handshake recebido QoS2 ja
iniciado nao e segura nesta versao. Aceite: queda depois de PUBLISH QoS2 e antes
PUBREL/PUBCOMP falha explicitamente, sem reconectar ou prometer exactly-once.
No --count, aguardar conclusao QoS2 por deadline fixo8s, nao prolongado por ping.
Adicionar fixture de queda apos PUBREC (sem replay), teste de cancelamento na
espera progressiva, documentar limitacao e matriz. Sem novo codec/dependencia.
