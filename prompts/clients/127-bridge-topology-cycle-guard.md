# C127 — guardião de ciclos por manifesto de topologia

Entrada: [P225](../../../xmqr/prompts/broker/225-bridge-topology-cycle-guard.md). Este prompt precede o código.

Implementar `topology_file` privado e `link_id` em TOML v1 da bridge. O manifesto lista links com IDs únicos, origem/destino com identidade estável de broker, prefixos e QoS. Antes de conectar, validar a correspondência exata da rota local e formar grafo de pares `(broker_id, prefix)`; recusar caminho que retorna ao mesmo par e qualquer ciclo no conjunto. Recusar aliases/duplicidades ambíguas, arquivo ausente ou permissões excessivas. Manter QoS 0 legado sem manifesto; exigir manifesto para QoS 1, com mensagem de migração clara.

Testar ciclos de 2/3 links, cadeia acíclica real de duas bridges e destino final, mismatch, segredo/caminho inválido, regressão QoS 0 e QoS 1, CI. O manifesto cobre somente o domínio gerenciado; documentar ausência de traço por mensagem no MQTT 3.1.1 e bridge externa fora do controle. Não marcar fase 8 concluída apenas com esse guardião.
