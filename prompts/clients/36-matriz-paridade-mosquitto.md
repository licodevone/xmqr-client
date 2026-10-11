# C36 rev1 — matriz de paridade Mosquitto

Entrada: client HEAD98b72f0, árvore limpa; broker4fe905a com P55 local em validação Unix pendente. Pedido atual autoriza evolução por prompts e tags de marcos aceitos, releases quando apropriadas. Aplicar skill xmqr-client-development. Serena em diagnóstico, não alegar conexão.

Objetivo: documentar opções existentes e lacunas perante mosquitto_pub/sub e preparar testes de limites conforme fase1. Escopo documental: docs/mosquitto-parity.md, README, AGENTS e registroC36. Não alterar protocolo, limites, credenciais, versão ou funcionalidades nessa fatia.

Referência: manuais oficiais Mosquitto online; registrar data e distinguir opções2.1. Matriz verificável com opção XMQR, evidência no código/testes e próximo incremento. Documentar teste cruzado XMQR client/Mosquitto e Mosquitto clients/XMQR com TLS/QoS/session/retained/Will e limites; não atribuir PASS sem execução.

Aceite: links locais válidos, leitura CLI/session/wire, distinção do pacote0.5.0 e código posterior, status explícito dos gates externos. Tag anotada de marco documental c36-paridade-v1 após commit exato e validação documental; não é tag de versão de pacote. Próxima funcionalidade C37 terá prompt próprio de timeout/keepalive e tag vX.Y.Z após gates. Nenhuma release só por esta matriz.
