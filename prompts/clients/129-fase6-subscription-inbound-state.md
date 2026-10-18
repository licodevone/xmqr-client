# C129 v1.0 — journal inbound da assinatura persistente MQTT 5

18/10/2026, preparado antes do código. Pedido: fechar M03/6B do plano de
paridade e a pendência explícita do prompt C110, coordenado pelo broker P227.
Entrada: clone local próprio limpo b9f46cb49646335bcb93ffa769eb2042796524b7,
client publicado0.36.0, broker0.42.0/admin0.10.0. C128/bridge em edição no
repositório original é alheio; não importar/reverter/commitar seus arquivos.

## Escopo e decisões

Ownership: src/outbox.rs, src/v5_session.rs, tests/wire.rs, novos testes
dedicados e vendor/rumqttc-0.25.1/src/v5/{state.rs,client.rs}, mais
vendor/rumqttc-0.25.1/XMQR-PATCHES.md. Sem dependência nova, Cargo/Docker/CI,
README/CLI/bridge alterados. --subscription-state-dir existente é o opt-in,
Linux/WSL, mesmo SubscriptionSpec/destino/identidade/filtros e SessionPresent1.
Preservar outbox de publicação C105/C107, MQTT3.1.1 e modo sem manifesto.
IDs novos/flags/nomes de usuário não serão presumidos. O coordenador integra
docs globais e distribuição depois dos gates; não editar repositório original.

Publisher continua serial, quota inicial1 é permitida por MQTT4.9 e não exige
pipeline por performance. Backlog/limites bilaterais do broker serão testados
separadamente, sem retirar requisito existente. Journal inbound contém no
máximo32 registros e65536 bytes serializados, incluindo metadados, no diretório
0700 e arquivos0600 com proprietário/lock/no-follow/hash/fsync/rename existentes.
Quota cheia ou armazenamento incerto fecha sem ACK, preservando a obrigação
do broker. Não aumentar silenciosamente quotas ou prometer produção.

## Estados e falhas

Received: pacote QoS1/2 resolvido (sem alias transitório), payload binário,
metadados/propriedades e identificador validados e duráveis antes da saída e
de qualquer ACK. OutputReady: depois de saída/flush ou filtro, persistir a
transição antes do PUBACK/PUBREC. QoS2 segue AwaitPubRel; ao receber PUBREL,
persistir PubCompPending antes de enfileirar PUBCOMP. Limpar registro após
evento Outgoing do ACK terminal; erro de limpeza permanece explícito.
Reabrir Received pode repetir stdout: queda entre flush e commit não é uma
transação com o consumidor. Não alegar exatamente uma vez de aplicação.
Erro de saída não confirma mensagem; payload continua recuperável no journal.
Duplicatas da mesma obrigação não repetem saída de uma fase já confirmada;
identificador reusado após conclusão inicia nova publicação, inclusive DUP.
Sem SessionPresent ou especificação divergente, preservar todos os arquivos
e falhar antes de saída/replay/ACK. Sem segredos ou payloads nos diagnósticos.

Fork: modo explícito de incoming QoS2 controlado pelo journal, ativado apenas
na assinatura opt-in. Evitar PUBCOMP automático e rejeição prematura de PUBREL
após restart por bitset transitório vazio. Encaminhar PUBREL ao caller que
valida journal, completa duravelmente e enfileira PUBCOMP manual; desconhecido
recebe motivo0x92 sem saída falsa. Preservar comportamento padrão e todos
copyrights/textos Apache-2.0 originais; registrar patch sem relicenciamento.

## Regressões e aceite finito

Antes do fix, executar reprodução dirigida no ELF antigo da assinatura que
crasha após PUBREC e retoma com SessionPresent1/PUBREL: exigir conclusão,
continuidade e ausência de saída duplicada nesse caso. Manter RED real.
Unitários de roundtrip/fases/checksum/limite32/64KiB/lock/symlink/permissões;
fixtures TCP de QoS1/2 e recuperação, ACK após commit, saída falha, filtros,
crash após PUBREC/antesPUBREL, deadline enquanto aguarda PUBREL, PUBREL repetido,
SessionPresent0, declaração divergente e arquivo corrompido sem replay/ACK.
Repetir C105/C107/C108/C110 e WSS quando pertinentes em recursos sintéticos.
Não simular perda de energia por SIGKILL nem usar serviços/estado do operador.

Gates locked fmt/test/Clippy/build/inventário, Windowscheck proporcional,
fonte exata/hash/ambiente e resultados reais em registro C129. TMPDIR=/var/tmp,
cache privado -j1; não copiar target/cache inteiro nem instalar ferramentas.
Prompt revisado antes de correções após FAIL. Sem tag/push/publicação pelo
agente; commits locais autorizados para isolar a contribuição. Coordenador
decide integração e release após aceite e regressões finais.

Norma primária consultada: OASIS MQTT5 §§3.2.2,4.1,4.3.2/4.3.3,4.4,4.9:
https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html
