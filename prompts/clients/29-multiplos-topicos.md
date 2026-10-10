# C29 — assinatura de múltiplos tópicos, candidato 0.4.0

Prompt 1.0, 2026-10-10. Estado: preparado antes do código. Pedido autorizado:
implementar a próxima melhoria delimitada do client. Versão independente 0.4.0.

## Estado de entrada

`xmqr-client` 0.3.0, tag local/remota `v0.3.0`, HEAD `d26eb9c`; branch `main`
alinhada com `origin/main`, sem alterações prévias do usuário. Broker permanece
independente em `../xmqr`, versão 0.8.0. A CLI rumqttc aceita hoje um `--topic`
por execução e aplica o mesmo QoS de assinatura a esse filtro.

## Objetivo e escopo

Permitir vários filtros MQTT em `sub` por repetição de `--topic`, mantendo a
forma atual com um filtro. Exemplo:

```text
mqtt-client sub --open-lab --topic 'sensor/+/value' --topic 'alerts/#' --qos 1
```

Em `pub`, `--topic` continua obrigatório e único. Em `sub`, exigir de 1 a 256
valores; validar cada filtro UTF-8/NUL/gramática e limite de 1024 bytes. Rejeitar
filtros duplicados para evitar ambiguidade no SUBACK e saída. Um único `--qos`
solicitado vale para todos. Calcular o tamanho real do pacote SUBSCRIBE codificado
antes de conectar e rejeitar pacote acima de 64 KiB; não truncar nem dividir em
várias assinaturas. `--count`, reconexão, JSONL/texto, sessão, TLS/SAN, credenciais,
Will, payload binário e demais modos preservam o comportamento atual.

Representar os filtros como lista na configuração validada. Enviar uma solicitação
SUBSCRIBE contendo todos eles. Em sessão nova, sessão ausente após reconnect ou
pedido anterior não confirmado, reenviar o lote uma vez. Em sessão retomada já
confirmada, preservar a assinatura sem duplicar SUBSCRIBE. SUBACK só é aceito se
trouxer exatamente uma razão por filtro e cada concessão corresponder ao QoS
solicitado; falha parcial ou vetor inesperado encerra com erro explícito.
Limites de mensagens continuam contados no total de todos os tópicos. A saída
continua identificando o tópico de cada mensagem.

Não adicionar sintaxe de lista separada por vírgula, wildcard novo, QoS individual,
perfis de conexão, limite de duração, timestamp, nova dependência ou recurso no
broker. Não alterar o formato JSONL existente nem prometer que o cliente recebe
mais do que o broker/sessão autorizam.

## Arquivos previstos

`src/cli.rs`, `src/session.rs`, testes de CLI/wire, README, CHANGELOG, índice e
matriz de cobertura, e registro C29. Cargo.toml/Cargo.lock apenas para alinhar a
versão independente `0.4.0`, sem upgrade de dependências.

## Aceite e validação

- `pub` aceita exatamente um tópico e rejeita repetição; `sub` aceita um ou vários,
  rejeita zero/duplicados/mais de 256, cada filtro inválido/over-limit e pacote
  acima de 64 KiB antes de iniciar rede.
- SUBSCRIBE/SUBACK cobrem vetor completo, falha de um filtro, tamanho de vetor
  incorreto, QoS, reconexão sem sessão, retomada de sessão e `--count` agregado.
- Integração com broker isolado verifica mensagens em filtros disjuntos/overlap,
  deduplicação de entrega pelo broker, retained e wildcard `$` sem alterar ACL.
- Gates: `cargo fmt --all -- --check`, `cargo test --locked`, `cargo clippy
  --locked --all-targets -- -D warnings`, `cargo build --locked`; validar Windows
  e Ubuntu-26.04/WSL com broker e estado de teste próprios. Ferramenta ausente é
  BLOCKED; não instalar Mosquitto.

Registrar resultados reais, PASS/FAIL/BLOCKED e limites. Candidato `v0.4.0` não
é tag criada por este trabalho. Sem commit, push ou publicação.
