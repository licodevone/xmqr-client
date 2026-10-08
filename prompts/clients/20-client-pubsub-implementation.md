> **Adaptação XMQR — 2026-10-07.** Histórico de escopo da origem, sem prova de execução no XMQR. Não reaplicar sobre o código existente. Para validar o snapshot atual, usar broker/36 e clients/24.
> Leia primeiro [o contrato comum](../contrato-base.md) e a matriz [de cobertura](../COBERTURA.md).
> Origem literal preservada: [arquivo original](../origem/clients/20-client-pubsub-implementation.md). Nenhum agente, skill ou MCP externo é requisito instalado.
> O contrato comum prevalece sobre instruções históricas de versão, ambiente e execução. Leitura não autoriza implementação nem execução automática.
# Prompt — implementação do cliente Rust pub/sub mTLS

Implemente a fatia aprovada em `prompts/clients/19-client-pubsub-scope.md` para MQTT **3.1.1, QoS 0**, preservando o código e os certificados existentes. Edite os parâmetros se o escopo tiver mudado.

## Contrato desejado

- CLI: `pub --topic <nome> --message <texto>` e `sub --topic <filtro>`, com `--host`, `--port`, `--ca`, `--cert`, `--key` e `--client-id` explícitos ou defaults de laboratório documentados. Evite argumentos que revelem segredos; nunca ofereça `--insecure` nesta fatia.
- TLS: use `tokio-rustls`/`rustls`, valide cadeia pela CA fornecida e SAN/hostname ou IP do destino. Carregue certificado e chave do cliente para mTLS. Falhe fechado quando arquivo, cadeia, chave ou nome não corresponderem; não confunda o endereço de conexão com o nome TLS esperado.
- Protocolo: `CONNECT` com nível 4 e Clean Session, aguarde `CONNACK` válido antes de publicar/assinar. `pub` envia `PUBLISH` QoS 0 sem Packet Identifier e encerra com `DISCONNECT`. `sub` envia `SUBSCRIBE` QoS 0 com Packet Identifier não zero, verifica `SUBACK`, recebe `PUBLISH` fragmentados/concatenados e mantém Keep Alive via ping ou escolhe valor compatível com a sessão e documenta a decisão.
- Segurança de recursos: limite de comprimento de pacote **antes** de alocar, de tópico/filtro e payload, tempos máximos para TCP/TLS/CONNACK/SUBACK/leitura/escrita e saída limitada do assinante quando configurada. Falha de rede não deve gerar loop sem limite nem panic. Não use bloqueio ou lock mantido durante `.await`.
- Estrutura: transporte, codec mínimo, estados do cliente e apresentação CLI separados por módulos/contratos claros. Reutilize um codec existente somente após confirmar sua correção e adequação; não acople cliente à lógica interna do broker.

## Sequência

1. Leia as instruções locais, se existirem, os prompts 19 e 21, os agentes/skills de arquitetura, protocolo e segurança e a especificação OASIS. Registre decisões que alterem API ou formato em ADR quando aplicável.
2. Faça uma tabela de estados/pacotes: entrada, estado anterior, validação, saída no wire, próximo estado e seção OASIS. Rejeite respostas inesperadas e códigos de erro; trate EOF e timeout separadamente.
3. Implemente em fatias compiláveis: (a) TLS e `CONNECT`/`CONNACK`; (b) `pub` QoS 0; (c) `sub` QoS 0 e entrega de mensagens; (d) encerramento/Keep Alive. Não anuncie sucesso de publicação QoS 0 como confirmação de entrega pelo broker.
4. Adicione testes unitários de bytes conhecidos e validação negativa; teste integração contra servidor simulado e um broker externo. Não altere `src/main.rs` para mascarar ausência de protocolo sem autorização explícita para a fatia do servidor.
5. Execute `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test`; registre comandos e resultados. Atualize a matriz de conformidade apenas para comportamentos efetivamente observados.

Entrega: arquivos alterados, comandos `pub`/`sub` com caminhos PEM, evidências de teste, limitações e próximo passo para integrar o broker. Nunca declare interoperabilidade MQTT completa com base no round-trip do próprio cliente.
