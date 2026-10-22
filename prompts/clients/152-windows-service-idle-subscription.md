# C152 - Assinatura persistente no servico Windows

Versao1.6 - 21/10/2026 America/Sao_Paulo.

Complemento de empacotamento P248: runtime C152100testes/fmt/clippy/release
passaram. Copia PE modificada apos compilar passou auditoria mas foi recusada
ao executar e desapareceu; causa externa nao comprovada. Incorporar ICOClient
por compilador de recursos WindowsSDK no link nativo via build.rs/asset do
manifesto windows-service, sem deps novas/RUSTFLAGSglobais. Verificador readonly
do Brokerconfere nove imagens/recursos. Reexecutar fmt/clippy/release/probe,
sem mudar runtime/credenciais, protecoes do host ou instalar automaticamente.

Revisao1.5: TMP/TEMP privado novo resolveu os tres erros ambientais; suite
completa100/100 PASS em0.36s, sem alterar testes antigos ou validadores.
Clippy estrito encontrou duas conversoes usize->u8 no novo frame QoS2 de
teste; trocar por try_from e repetir Clippy/fmt/testes/finalrelease.

Revisao1.4:11/11 testes de sessao corrigidos PASS; suite de servico completa
97PASS/3FAIL por acesso negado5 em escritas de fixtures privadas preexistentes.
Nao enfraquecer validadores nem alterar testes antigos para ocultar permissao.
Reexecutar com TMP/TEMP dedicados novos como no C151/P242 e registrar resultado.
Clippy deve usar incremental desabilitado para evitar warning de hardlink do
ambiente. Se bloqueio permanecer, separar evidencia C152 dos gates ambientais.

Revisao1.3: testes compilaram; caso CLI expirou em91s virtuais mas falhou por
asserir uma mensagem especifica entre dois timeouts simultaneos existentes.
Fixture combinado total/retry ficou bloqueado; separar o controle de prazo
total em tarefa independente com deadline absoluto e cancelamento explicito,
sem borrow manual de future. Validar classe de erro de timeout, nao vencedor
de corrida diagnostica. Interromper somente o processo de teste proprio.
Uma tentativa de consulta do processo falhou antes de criar shell porque
exec-server transport desconectou; registrar bloqueio de ambiente se persistir.

Revisao1.2: release nativo passou em17.19s com cache curto xp242-cargo.
Primeira compilacao de testes encontrou E0716 apenas nos arrays temporarios
do fixture tokio::select; dar bindings locais aos filtros antes de repetir.
Nao alterar runtime para acomodar o fixture; testes usam peer TCP loopback
isolado e avanco virtual de180s, sem wait real90s e sem SCM/credenciais.

Revisao1.1: o escopo de sessao de servico inclui SCM ativo OU sink privado
ativo de --service-config em console, somente em Windows com feature
windows-service-runtime. Isso permite prova isolada do mesmo caminho sem SCM;
CLI comum continua limitada a90s. Representar ausencia de prazo idle como None,
mantendo prazos CONNECT/SUBACK/QoS2 e total explicito separados. Permitir
tokio test-util apenas como dependencia de desenvolvimento para tempo virtual.
Base44e13a4b, versao0.43.0; arvore limpa antes desta fatia.

## Evidencia e objetivo

P247 result2026-10-22T02:04:52Z confirma tres servicos Running apos preparo.
Broker /ready200 e Admin /api/auth/config200 confirmados pelo agente.
Broker metricas mostram uma conexao total/uma sessao e zero conexoesativas,
sem rejeicao de auth/ACL. Leitura do codigo confirma subscriber MQTT3.1.1
encerra por IDLE_TIMEOUT90s mesmo com PINGRESP. Isso e politica CLI existente,
mas nao atende o servico persistente preparado sem count/total-timeout.
Nao afirmar estado SCM atual do Client sem nova coleta elevada; ferramentas
tem SCM/CIM/GetProcess negados ou resultados filtrados.

## Escopo autorizado

Corrigir somente modo Windows-service MQTT3.1.1 subscriber para permanecer
aguardando mensagens sem limite de inatividade de aplicacao. Preservar
timeout de CONNECT/SUBACK, QoS2, timeout total explicito, reconexoes limitadas,
controle Stop/Shutdown cooperativo, persistencia/output8MiB e TLS/SAN/ACL.
CLI normal conserva semantica90s, sem mudar opcoes/versoes/protocolo publico.
MQTT5 nao e ampliado por esta fatia; documentar alcance.

Arquivos: src/session.rs e testes pertinentes, docs, registroC152, manifesto
dev se indispensavel para testar tempo virtual. Sem tags/commits/push/MSI.
Build do manifesto packaging/windows-service em target existente, lock/cache
locais; nunca trocar binario instalado automaticamente. Entregar exe atual
recompilado e identificado, se build possivel; se bloqueado relatar causa.

## Validacao e aceite

Provar espera alem de90s para servico e expiracao CLI, protocolo limitado e
cancelamento, sem esperas reais longas quando teste virtual equivalente.
fmt, testes locked, clippy all-targets -Dwarnings e release Windows nativo
com runtime servico. Nao gerar nem copiar credenciais reais. Nenhuma alteracao
SCM/ProgramFiles/ProgramData/ACL. Integracao com estado real somente por
operador elevado em etapa posterior com backup do binario e verificacao.
