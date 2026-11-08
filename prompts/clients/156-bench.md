# C156 v1.1 — benchmark MQTT do Client

Base `312ea86` (candidato 0.44.0). Adicionar `mqtt-client bench` sem
absorver a mudança na versão congelada. Reutilizar credenciais, perfis,
TLS/SAN e motor do probe em `src/probe.rs`, com coordenação em `src/bench.rs`.

Modos `conn`, `reconnect` e `echo`, até 1.000 clientes, prazo limitado,
identidades efêmeras e sessões limpas. Manter conexões com event loops ativos;
reconexão abre duas ondas após fechar a primeira. Echo usa tópicos sintéticos
isolados, QoS 0 sem retenção e confirmação pelo marcador. Medir contagens,
vazão observada e percentis de conexão/entrega em JSON redigido. Sem mudanças
na instalação, estado ou versões. Testar limites, cálculo, CLI e rede isolada.
Registrar evidência em `prompts/registros/C156-bench.md`.

Revisão 1.1. O Clippy direto detectou distribuição do bound, pontuação de
bloco e variante de opções grande. Unificar o bound, ajustar a instrução
e alocar separadamente as opções dos dois protocolos. Repetir o Clippy e o build direto.
