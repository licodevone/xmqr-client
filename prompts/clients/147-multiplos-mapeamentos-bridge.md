# C147 v1.0 — múltiplos mapeamentos e diagnóstico da bridge

Data: 2026-10-19. Entrada `6baeeec`, client 0.42.1/bridge 0.4.0 publicados;
Git limpo e builds locais ignorados. Pedido atual: implementar toda a fase 8,
contrato P235 no broker. Esta é a primeira fatia, não seu aceite global.

Objetivo: preservar TOML v1 e acrescentar TOML v2 com 1–8 `[[routes]]`, cada
uma com `id`, `source_prefix`, `target_prefix` e QoS 0/1/2. Endpoints, protocolo
de origem, manifesto e tentativas são comuns; identidades MQTT e diretórios de
estado são derivados por rota. Não iniciar rotas parcialmente validadas.

Escopo: módulo de configuração/orquestração da bridge, pequeno ajuste do binário
e testes/fixture de integração. `--check --config ARQUIVO` e a ordem equivalente
validam arquivos/segurança/topologia e materiais TLS sem conectar nos brokers
nem criar diretórios. Diagnóstico não imprime credenciais, payload ou caminhos.
Uma falha fatal de worker encerra o conjunto, sem fingir funcionamento completo.

Regras: `topology_file` obrigatório no v2 e cada `route.id` corresponde ao link
exato no manifesto. IDs únicos/válidos e prefixos de origem não sobrepostos;
recusar mistura route/routes, teto, arquivos permissivos e nomes derivados
acima de 128 bytes. TLS/mTLS, ACL, limites de tópico/payload e v1 preservados.
`state_dir` privado/absoluto é obrigatório se houver QoS2; filhos privados por
rota são criados somente ao executar, sem seguir symlinks. Não apagar estado.

Cada rota é uma nova perna de publicação no QoS configurado. Mensagens da
origem podem ter QoS menor que a assinatura, e não devem derrubar a bridge:
ACK/journal da origem seguem o QoS efetivamente recebido; QoS0 na origem não
ganha garantia retroativa por ser republicado com QoS1/2. Nunca gravar PID zero
como obrigação QoS2. QoS recebido maior que o negociado é erro.

Aceite: testes positivos/negativos de schema/IDs/overlap/tetos/estado privado,
identidades estáveis e --check sem conexão/mutação. Dois mapeamentos reais
simultâneos, QoS mistos na origem, isolamento de tópicos/estado, falhas de ACL/
configuração e regressões dos modos v1. fmt/test/Clippy/build/inventário e CI
da fonte exata. Registro com PASS/FAIL/BLOCKED, guia atualizado e commit/push.
Sem bump/tag nesta fatia; distribuição e matriz global são etapas posteriores.
