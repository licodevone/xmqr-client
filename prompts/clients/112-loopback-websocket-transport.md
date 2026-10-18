# C112 — transporte WebSocket de laboratório no client

Entrada: client independente 0.31.0 publicado, broker 0.38.0 publicado, fase 7 global aberta. O fork rumqttc 0.25.1 contém suporte opcional a WebSocket, ainda não habilitado no XMQR. Git limpo no início; este prompt precede qualquer alteração funcional.

Objetivo: adicionar `--ws-url ws://127.0.0.1:PORTA/CAMINHO` (ou `[::1]`) para `pub` e `sub` nos protocolos 3.1.1 e 5, somente com `--open-lab`. Habilitar feature `websocket` do fork e selecionar `Transport::ws()` sem alterar TCP/TLS existentes. Validar URL antes de rede: esquema `ws`, host IP loopback, porta positiva, caminho não vazio, sem usuário/senha, query ou fragmento; rejeitar `--host`/`--port` explícitos junto à URL. Exigir subprotocolo MQTT na resposta do servidor via validação já existente do fork. Nenhuma credencial no argumento ou log.

Escopo: `Cargo.toml`/lock, CLI, configuração MQTT 3.1.1/5, documentação e testes. Sem WSS neste recorte; WSS seguro exigirá prompt posterior com CA/SAN/mTLS e ensaio próprio. Não alterar broker/admin, perfis nem estado persistente.

Revisão de escopo antes do código: recusar `--outbox-dir` e `--subscription-state-dir` com `--ws-url` neste primeiro recorte. Os manifests atuais identificam host/porta TCP, não URL/caminho WebSocket; aceitar a combinação permitiria retomar uma operação em destino diferente. A integração durável com URL exigirá migração/versão própria. `--ws-url` fica somente na CLI explícita, sem expansão de perfil por enquanto.

Aceite: negativos da CLI para URL externa, esquema errado, userinfo, query, fragmento, porta ausente/zero, conflito de destino e modo seguro; testes de seleção do transporte nos dois protocolos; fixture WebSocket local com subprotocolo `mqtt`, publicação/assinatura e recusa sem subprotocolo, mais regressão TCP/TLS. Fmt, suíte, Clippy, build, licenças/distribuição e CI da fonte exata. Registrar resultado parcial no checklist; fase 7 permanece aberta até WS/WSS broker+client e regressões completas. Sem tag/release isolada.

Revisão 2 após teste inicial: o unitário de validação da CLI passou, mas a primeira fixture WebSocket fechou o socket logo após receber o PUBLISH QoS 0 e o client relatou `Connection closed by peer abruptly`. O cenário não demonstra conclusão válida da publicação. Corrigir a fixture para manter o WebSocket até o DISCONNECT MQTT (e fechar pelo handshake WebSocket), identificar o protocolo/caso se falhar de novo e repetir os gates; não registrar o primeiro resultado como PASS.

Revisão 3: após esperar o DISCONNECT, os cenários de publicação deixaram de falhar, mas a fixture de assinatura exigiu incorretamente que `stdout` terminasse em `hello\n`. O processo saiu com sucesso; capturar a saída real e afirmar somente o conteúdo observável documentado, sem afrouxar a checagem do payload. Repetir todos os cenários.

Revisão 4: a suíte completa (88 unitários, 66 wire e um teste WebSocket) passou, mas Clippy `-D warnings` falhou em `tests/websocket.rs` com `result_large_err` na assinatura imposta pelo callback de handshake do tungstenite. Tratar somente esse aviso no menor escopo com justificativa explícita, sem relaxar o gate do projeto, e repetir Clippy e os testes pertinentes.
