# C148 — encaminhamento MQTT 5 durável e percurso

Prompt [C148v1.15](../clients/148-bridge-mqtt5-duravel-percurso.md), original
publicado em2200ad7 antes do código; revisões registram falhas antes de corrigi-las.
Entrada C1473350bd2, client0.42.1/bridge0.4.0 publicados. Este registro aceita
a implementação local; CI exata e distribuição C149 ainda pendentes.

## Resultado local em19/10/2026

- Gates fmt/test/clippy(-Dwarnings)/build/inventário PASS, sem dependência nova.
  Testes Rust:208PASS (15bridge,112CLI,7credenciais,5perfis,1TLS,68integração).
- Mesmo ELF bridge SHA256
  `5686c2ddfb7304f3f5a02a63e6d43f3321f99f861bb6ca9093737fa2d52c7299`:
  wire33PASS/0FAIL/1NOT_RUN; recuperação15/15PASS; C147regressão23/23PASS;
  brokers reais34/34PASS. Relatórios em[docs/conformance/c148](../../docs/conformance/c148/).
- XMQR0.43.0↔XMQR e XMQR↔Mosquitto nos dois sentidos; imagem Mosquitto2.0.22
  digest`sha256:199ea8ef2e35ec2b1b37e59cfd1dbae538ed4dfa4a2251a121a52215a6248a21`.
  Três rotas, seis combinações de QoS por direção, propriedades ordenadas,
  retained inicial/vivo/tombstone, TTL descontado, journals isolados, cadeia
  de três brokers, remapeamento de resposta, retorno cooperante/teto de saltos.
- NOT_RUN wire: alias/subID não foram negociados legalmente. Não se injetou
  pacote inválido para alegar prova positiva; transformação Publish→Envelope
  tem teste unitário próprio removendo os dois campos locais à conexão.
- Crash cobre fases Prepared/Sent/Rel/Done, origem PUBREC/PUBCOMP, sessão
  ausente, estado corrompido/incompatível/identidade/relógio/limite/symlink,
  exclusão mútua,0600, counters/overflow, expiração durante fsync e ACKs
  negativos conhecidos persistidos em Refused, recusando reabertura preconnect.

## Falhas preservadas e correções

Os relatórios before e wire-final01 preservam falhas reproduzidas de runtime:
identidade por perna, SIGINT durante await,0600 exato, counters, TTL expirado
antes de enqueue, QoS2PID apósPUBCOMP, recusa negativa e escrita redundante
da identidade conhecida. O prompt documenta também falhas Clippy/compilação
e suas revisões; não se converteu resultado antigo em PASS.

Uma versão anterior do oráculo QoS2 estava incorreta: depois de PUBCOMP,
PUBLISH com PID reutilizado é mensagem nova mesmo com DUP. O resultado antigo
foi invalidado; quatro combinações PID/DUP/conteúdo e duplicata pendente foram
reexecutadas com o oráculo normativo correto.

Interop real01 falhou antes dos cenários: fixture usava concessão publish `#`,
recusada pelo broker que exige nomes exatos. Real02 falhou no SUBACK: também
era necessário declarar exatamente os filtros assinados. O prompt foi revisto
antes de corrigir só as ACLs descartáveis; real03 concluiu34/34. Nenhuma
permissão real foi ampliada. Timeout do leitor TTL também foi preservado como
inconclusivo antes de corrigir sua resposta multiplexada a PINGREQ.

## Contrato e limites

Modo v2 opt-inMQTT5 nas duas pontas,1–8rotas, estadoformato2 e isolamento por
rota. Metadados/prioridades de resposta explícitos; TTL absoluto não renovado,
com descarte conservador quando resta menos de1s antes de nova entrega.
Descarte e contador são duráveis. Um voo por rota; burstsQoS0 podem exceder
cota e encerrar grupo. Sem commit atômico distribuído: resultado remoto incerto
pode duplicar. Marcador só protege intermediários que o preservam; cenário
finito documenta sua remoção, sem prometer proteção universal.

Draftv2MQTT5→3.1.1 exige `legacy_metadata_policy="drop"`; v1 mantém contrato
histórico. Formato novo exige diretório/IDs novos e migração/rollback após
drenagem. Journal Refused não tem comando de desbloqueio nesta fatia: preservar
obrigação e investigar offline. Não apagar estado para reiniciar.

[Guia e migração](../../docs/bridge-mqtt5.md). Sem instalar pacotes no host,
alterar serviços reais ou usar credenciais do operador. Recursos temporários
próprios encerrados. Fase8/P235 depende ainda da distribuição e reconciliação
global; os resultados locais não são certificaçãoMQTT5 integral.
