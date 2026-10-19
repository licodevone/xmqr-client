# C148 v1.15 — encaminhamento MQTT 5 durável e percurso limitado

Data: 2026-10-19. Entrada publicada client 0.42.1/bridge 0.4.0; C147 aceito
em3350bd2, CI39271798362 PASS e21/21 cenários de múltiplos mapeamentos.
Pedido: prosseguir até concluir a fase 8. Contrato global P235 preservado.

Objetivo: modo v2 opt-in `destination_protocol="mqtt5"`, com MQTT 5 nas duas
pontas, preservação de metadados, journal de encaminhamento e defesa de percurso.
Não alterar o engine legado v1/v2 com destino 3.1.1 nem importar o broker.

Escopo: módulos pequenos de metadados, armazenamento e execução da bridge,
configuração v2, testes unitários/fixture wire e integração mTLS/interop, guia,
CI e registro. Sem nova dependência. Prompt precede toda edição funcional.

Configuração: exigir origem MQTT 5, `state_dir` privado mesmo em QoS0,
`bridge_id` estável (até 24 bytes ASCII seguros), `max_hops` entre 1 e 8 e
`response_topic_policy` explícita (`preserve` ou `map-prefix`). Os novos campos
só se aplicam ao destino MQTT 5. `--check` recusa combinações incoerentes e
valida sem abrir conexões/mutar estado. Identidades e filhos continuam por rota.

Metadados: preservar PFI, Content Type, Response Topic conforme política,
Correlation Data binária e User Properties ordenadas/repetidas. Não copiar
Topic Alias nem Subscription Identifier. Validar UTF-8 quando PFI=1. Expiry
vira prazo absoluto privado; descontar tempo antes de cada envio/retomada, sem
renovar TTL. Expirado é descarte explícito e contabilizado, não entrega alegada.
Relógio regressivo falha fechado preservando estado. Limites: tópico1024,
payload64KiB já existente, até 16 User Properties incluindo marcador, campos
limitados a 256 bytes, pacote respeita Maximum Packet Size do destino. Não
reduzir QoS nem remover retained quando CONNACK os proibir. ACK negativo falha
sem confirmar origem. PUBLISH sem metadados também funciona.

Percurso: propriedade reservada `xmqr.bridge.path`, uma única ocorrência,
lista limitada de bridge IDs (até oito). Validar caracteres, teto e unicidade;
adicionar identidade deste processo (comum às suas rotas), preservando as demais
propriedades. Retorno à própria bridge e teto de saltos resultam em descarte
durável/ACK da origem, sem novo encaminhamento. Marcador inválido é erro sem
ACK. Manter manifesto acíclico como segunda defesa. Não há proteção garantida
se um intermediário apagar propriedades; documentar/testar essa fronteira.

Durabilidade: arquivo separado `bridge-forward.json`, formato versionado,
lock exclusivo, diretório0700/arquivo0600 e recusa de symlinks, limite2MiB.
Uma mensagem em voo por rota; payload/metadados/QoS/identidade/prazo/fases e
conclusões QoS2 limitadas persistidos atomicamente com fsync antes dos ACKs e
das transições correspondentes. Usar spawn_blocking para disco/hash/TLS.
Destino QoS1: PID estável/DUP após incerteza; QoS2: persistir PUBREC→PUBREL
antes de enviar PUBREL, retomar sem republicar payload após essa fase.
Origem QoS2: registrar obrigação antes de PUBREC e intenção antes de PUBCOMP;
PUBREL repetido não republica mensagem concluída; PUBLISH apósPUBCOMP com PID
reutilizado é nova mensagem independentemente deDUP. Não confirmar PUBREL desconhecido
como sucesso. Não apagar estado incompatível/corrompido nem tentar adivinhá-lo.
Estado pendente exige sessões correspondentes; sessão remota anterior exige
identidade local conhecida. Reconexões recriam os loops e usam o journal,
impedindo reenvio automático com TTL antigo; tentativas limitadas.

Migração/rollback: os journals antigos continuam no engine legado. O modo novo
usa diretório novo vazio: recusar presença de journal legado, sem migração
implícita. Drenar/parar a rota antiga, preservar seu diretório e usar IDs novos
na migração. Rollback exige drenar o modo novo e restaurar a configuração/IDs
antigos; nunca reutilizar estado ou sessões incompatíveis. Nenhum commit atômico
distribuído: queda após aceite remoto incerto pode duplicar entre pernas.

Aceite: gates Rust e inventário; wire independente para capacidades/ACKs e
metadados/expiração, crash antes/depois de ACK/PUBREC/PUBREL, corrupção/lock/
limites/sessão ausente, retained vivo/replay/tombstone, percurso/cadeia/retorno
externo cooperante e múltiplas rotas; XMQR↔XMQR e XMQR↔Mosquitto nos dois
sentidos. Preservar FAIL e revisar prompt antes de corrigir. Registrar hashes
e CI exata. Release só após matriz integrada e validação dos pacotes/imagem;
não marcar a fase concluída apenas porque o código compilou.

Revisão de implementação: recusar IDs de rota `.` e `..`, que escapariam do
filho previsto apesar de satisfazerem o conjunto ASCII. Expiry limita novas
entregas; depois de PUBLISH incerto, completar o handshake com prazo restante
zero se necessário, sem abandonar PID remoto em uso nem renovar validade.

Primeiro gate Clippy: cinco diagnósticos de estilo/tamanho de future, sem
falha funcional nos 12 testes da bridge. Preservado em
/var/tmp/c148-clippy-1.log. Corrigir os diagnósticos sem desabilitar o gate;
usar Box::pin para a future de execução e manter cenários negativos.

Segundo gate Clippy: dois diagnósticos adicionais nos testes de metadados
(assert de vetor vazio e inicialização Default). Corrigir e repetir o gate.
Em paralelo, o pedido de painel web é executado em A23 do admin; não é aceite
da fase8. Os testes wire/crash/interop C148 continuam pendentes.

Retomada v1.2 após pedido explícito de finalizar fase8: começar por fixture
TLS wire independente e depois crash/interop. Não declarar código existente
aceito por compilação. Scripts recebem binários explícitos e gravam relatórios
novos/sanitizados com hashes; recursos próprios e certificados descartáveis.
Auditar sessão/ACK por perna, QoS menor que assinatura, limite de um voo,
retenção viva/tombstone, identidade, prazo em replay e pacote recusado.

Primeira fixture wire falhou antes de TLS: configuração própria usava
max_retries=0, incompatível com limite1..20 já existente. Preservar relatório
parcial sanitizado e encerrar recursos próprios; corrigir apenas fixture para1
e repetir os mesmos cenários. Nenhuma falha de engine inferida desse resultado.

Antes das correções de runtime: fixture wire02 preservou28PASS/1FAIL no
encerramentoSIGINT/1NOT_RUN para alias/subID não negociados. Auditoria própria
/var/tmp/c148-audit-01.json reproduziu quatro falhas: destino aceito na primeira
tentativa não é persistido se origem falha; SIGINT durante await não encerra;
journal0400/0700 é aceito apesar do contrato exato0600; não há contadores
duráveis de descartes. Nenhum recurso do operador foi usado.

Correções autorizadas por este contrato: persistir identificação de cada
perna após seu CONNACK válido, antes de depender da próxima; manter um ouvinte
de CTRL+C durante todo run (inclusive conexão/disco/backoff), sem janela entre
selects; exigir0600 para journal/lock novos; contadores u64 de descartes por
expiry e percurso no mesmo commit que marca Done, sem dupla contagem após
replay. Overflow recusa semACK e preserva obrigação. Novo formato experimental
bridge-forward versão2, recusando arquivos de desenvolvimento versão1 sem
apagar/migrar implicitamente. Journals legados já publicados não mudam.

Repetir matriz wire, auditoria e checkpointsPrepared/Sent/Rel/Done/PUBREC/
PUBCOMP, negativos de armazenamento e integração entre brokers após rebuild.
Fixture de relógio precisa checksum válido para provar a recusa de clock,
sem confundir essa recusa com corrupção do checksum.

Primeiro build apósv1.4 recusou E0502: referência pending permanecia viva ao
persistir aceite do destino. Capturar os dois requisitos de sessão em booleanos
antes do commit e manter as mesmas verificações. Revisão independente também
identificou intervaloexpiry calculado antes do fsync/checkpoint: acrescentar
prova de atraso controlado antes de enqueue, antes de reparar essa janela.

Revisão normativa: [OASIS MQTT5 seção4.3.3](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html),
MQTT-4.3.3-10/12. Antes de PUBREL repetir PUBLISH pendente não duplica entrega;
após PUBCOMP todo PUBLISH com o mesmo PID é nova mensagem, mesmo seDUP=1.
Retirar aceite do cenário wire02 que premiou supressão apósPUBCOMP. Corrigir
primeiro o oráculo, preservarFAIL contraengine e remover supressão baseada nas
conclusões: elas existem só para responderPUBREL repetido apóscrash, não para
deduplicar novas publicações. Testar reusoPID com mesmo/diferente payload.

Precisão dos limites:256bytes aplica-se a ContentType,CorrelationData e
chave/valorUserProperty; ResponseTopic é nome de tópico e segue1024bytes,
inclusive após remapeamento. Não ampliar nem truncar esses limites.
Acrescentar unitário da transformaçãoPublish→Envelope com alias/subIDs de
conexão e metadados de aplicação, provando remoção/preservação e fingerprint.

V1.4 já passou11cenáriosde recuperação/auditoria, mas o aceite C148 continua
pendente destas provas adicionais. Corrigir expiry somente após prova do
intervalo commit→enqueue; recalcular prazo após espera local sem abandonar
handshakes já enviados/incertos.

Provaexpiry01 terminou TimeoutError inconclusivo: o helperreceive consumia
PINGREQ de uma perna e aguardava outro pacote nessa mesma perna, deixando
a segunda sem inspeção. Corrigir apenas leitura desse ensaio para responder
PINGREQ e voltar à multiplexação; preservar01 e repetir antes de fixTTL.

Provas antes do reparo: expiry02 recebeu novoPUBLISH após deadline expirar
durante checkpoint; wire-qos2-normative-before01 passou3casos e recusou2
de reusoPID/DUP1 (mesmo conteúdo ou diferente). Preservar ambos relatórios.
Remover supressão por conclusão apósPUBCOMP, mantendo supressão só do voo
pendente e recibos dePUBREL repetido. Recalcular propriedades depois do
commit/checkpoint, imediatamente antes de enqueue. Quando este processo sabe
que ainda não enfileirou o pacote e o prazo acabou, descarte durável; em
retomadaSent incerta comQoS>0, continuar handshake com expiry0 eDUP sem renovar
validade nem abandonarPID remoto. Quantização conservadora de segundos:
restante inferior a1s é descartado/contabilizado antes de uma nova entrega.

Primeira compilação do novo unitário de metadados recusouE0502: empréstimo
mutável das propriedades persistia durante fingerprint imutável do Publish.
Encerrar o empréstimo antes da comparação e retomá-lo só ao mutar depois;
manter todas as asserções da transformação e limites. Sem falha de runtime.

Os15unitários da bridge passaram; Clippy recusou assert_is_empty no novo
teste de SubscriptionIdentifiers. Usar assert_eq com vetor vazio para manter
diagnóstico do conteúdo, sem desabilitar lint nem retirar a asserção.

Wirefinal01:31PASS/2FAIL/1NOT_RUN. Todos cinco casos normativosQoS2 agoraPASS.
Sessão da origem ausente mudou saved_ms por commit redundante do destino já
conhecido: persistir destination_known só na transiçãofalse→true, preservando
bytes do journal se recusa posterior não trouxe informação nova. No negativo
de primeiro sourceSPdesconhecido, o oráculo antigo exigia ambosknownfalse;
deve exigir sourcefalse/destinationtrue pois o CONNACK válido do destino já
foi aceito/persistido. Manter asserções de obrigação/fase/payload/identidade e
sessões ausentes, sem aceitarACK precoce ou reenvio indevido.

Primeira integração entre brokers falhou na preparação de ACL da fixture:
publish=['#'] não é tópico exato aceito pela configuração do broker0.43.0.
Preservar relatório real-brokers1, corrigir apenas permissões de teste para
enumerar seus tópicos próprios; subscribe=['#'] continua válido. Não ampliar
ACL do broker nem tocar usuários/estado reais. Acrescentar linha/reason
sanitizados para falha externa de preparação, sem valores de credencial.

Auditoria adicional deMQTT-4.4.0-2 (mesma fonteOASIS): ACK>=0x80 termina o
PUBLISH remoto e não autoriza retransmissão automática dessa publicação.
Antes de alterar runtime, provar recusaPUBACK/PUBREC/PUBCOMP seguida de novo
processo. Se a recusa não estiver durável, persistir faseRefused antes de sair;
reabertura falha antes de conexões, semACKorigem e sem apagar obrigação.
Documentar bloqueio administrativo/ausência de retry transparente de recusas.
Não confundir negativo conhecido com conexão perdida/resultado incerto.

refusal-before01 reproduziu ausência de registro durável da recusa antes
do reparo. ImplementarRefused, testar os trêsACKs e reabertura sem conectar;
oráculos de negativos passam a exigir faseRefused em vezdeSent/Rel, preservando
as mesmas provas de ausência deACKorigem/novoPUBLISH e journal intacto.

Integraçãoreal02 falhou comSUBACK0x87: além dos tópicos exatos de publicação,
ACL de assinatura da fixture também exige filtros pedidos exatos. Enumerar
tópicos de observers e filtros das rotas/chains no bundle próprio, preservando
FAIL02 e sem ampliar permissões/configuração do broker do operador.

Para a fronteira de compatibilidade deP235: configuração v2 com origemMQTT5
e destino3.1.1 exige legacy_metadata_policy="drop" explícito. Recusar ausência,
valor desconhecido ou política legado em ambosMQTT5/source3.1.1. Configuração
v1 publicada mantém seu comportamento histórico documentado; modo v2 é novo
e ainda não distribuído. Acrescentar negativo de --check sem política, adaptar
somente exemplo/fixtureC147 v2 e documentar migração antes da primeira release.
Modo ambosMQTT5 continua preservando metadados, sem opção de descarte implícito.
