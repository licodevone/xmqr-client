# C148 v1.0 — encaminhamento MQTT 5 durável e percurso limitado

Data: 2026-10-19. Entrada publicada client 0.42.1/bridge 0.4.0; C147 está
em validação local, com 21/21 cenários de múltiplos mapeamentos aprovados.
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
repetições não republicam mensagem concluída. Não confirmar PUBREL desconhecido
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
