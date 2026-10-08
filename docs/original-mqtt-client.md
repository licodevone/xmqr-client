# Cliente de teste MQTT 3.1.1

Para a primeira aula, sem certificados, usuário ou ACL, siga o roteiro de três
terminais em [Laboratório inicial — MQTT aberto](laboratorio-mqtt-aberto.md).
Esse modo exige `--open-lab`, aceita somente loopback e usa QoS 0 inicialmente.
O mesmo roteiro evolui depois para usuário/senha e ACL, ainda sem certificados,
usando `--plain-auth-lab`.

O restante deste documento descreve o modo seguro posterior.

A série 0.5 do broker aceita filtros `+` e `#`, por exemplo
`--topic 'sensores/+/temperatura'`. `pub` exige um tópico concreto.
A ACL precisa conceder literalmente o filtro pedido; consulte
[filtros e testes externos](wildcard-subscriptions.md). O broker limita
tópicos/filtros a 1024 bytes UTF-8, com a mesma constante usada pela CLI.

O binário `mqtt-client` publica ou assina com QoS 0, 1 ou 2 sobre TLS mútuo **e usuário/senha MQTT**. Ele exige uma CA para validar o certificado e o nome/IP do servidor; não existe opção para desativar essa verificação. Os certificados abaixo são **somente para laboratório**. A identidade autorizada pelo broker deverá combinar o certificado do dispositivo, o usuário e a ACL do tópico.

## Compilar no Ubuntu/WSL

```bash
cd /caminho/para/xmqr
export CARGO_TARGET_DIR="$HOME/.cache/mqtt-broker-target"
cargo build --locked --release --bin mqtt-client
```

Antes de testar, configure o broker conforme [autenticação e ACL](security/auth-acl.md): no arquivo de usuários, vincule o SHA-256 de `client.crt` ao usuário escolhido e ao hash Argon2id da senha; na ACL, conceda `publish` e `subscribe` para `teste/mensagem`. `lab-device` abaixo é **apenas um exemplo**; substitua pelo nome configurado no broker. Não copie a senha para o comando: o cliente pedirá `Senha MQTT:` sem mostrá-la na tela. Cada terminal WSL solicitará a senha separadamente.

Deixe o broker rodando em um primeiro terminal WSL. Em um **segundo terminal WSL**, inicie a assinatura:

```bash
"$HOME/.cache/mqtt-broker-target/release/mqtt-client" sub \
  --topic 'teste/mensagem' --count 1 \
  --host 127.0.0.1 --port 1883 \
  --username 'lab-device' \
  --ca "$HOME/mqtt-lab-certs/ca.crt" \
  --cert "$HOME/mqtt-lab-certs/client.crt" \
  --key "$HOME/mqtt-lab-certs/client.key"
```

Em um **terceiro terminal WSL**, publique:

```bash
"$HOME/.cache/mqtt-broker-target/release/mqtt-client" pub \
  --topic 'teste/mensagem' --message 'Olá, MQTT!' \
  --host 127.0.0.1 --port 1883 \
  --username 'lab-device' \
  --ca "$HOME/mqtt-lab-certs/ca.crt" \
  --cert "$HOME/mqtt-lab-certs/client.crt" \
  --key "$HOME/mqtt-lab-certs/client.key"
```

Não é necessário entrar na pasta dos certificados, nem exportar `CARGO_TARGET_DIR` nos terminais de `sub` e `pub`: os comandos usam caminhos completos a partir de `$HOME`. O prompt `~` ou a pasta do projeto servem igualmente. `--host` precisa coincidir com o nome ou IP no SAN do certificado do servidor. O cliente valida a CA local, apresenta `client.crt`/`client.key` ao servidor e usa somente TLS 1.2/1.3. Também é possível definir os caminhos PEM por `MQTT_CA_CERT`, `MQTT_DEVICE_CERT` e `MQTT_DEVICE_KEY`. Não coloque chaves privadas ou senhas no repositório. Não existe opção `--password`, pois argumentos de linha de comando podem ficar visíveis no histórico e na lista de processos. Sem `--client-id`, cada execução gera um identificador novo; uma sessão persistente exige informar sempre o mesmo `--client-id` ao reconectar.

Para testes automatizados sem terminal interativo no Linux/WSL, acrescente `--password-file /caminho/privado/senha` a `pub` ou `sub`. Esse arquivo deve conter somente a senha (uma quebra de linha final é aceita), pertencer ao mesmo usuário que executa o cliente e ter permissão **0600**. O cliente rejeita arquivos legíveis por outros usuários, arquivos não regulares, senhas vazias e mais de 1024 bytes. Não use um arquivo dentro do repositório nem um caminho em `/mnt/c` ou `/mnt/d`, onde as permissões Windows/WSL podem não corresponder a `0600`. O modo interativo sem arquivo é o mais simples para os testes manuais acima.

`sub` permanece aberto até `Ctrl+C` ou até receber `--count N` mensagens. Os textos recebidos são mostrados com caracteres de controle escapados. Por padrão ambos usam QoS 0; acrescente `--qos 1` ou `--qos 2` a **ambos** os comandos para testar confirmações e entrega correspondentes. O publicador QoS 0 informa apenas envio; QoS 1/2 espera a confirmação do broker, que não equivale a confirmação da aplicação assinante.

Para guardar o último valor de um tópico, acrescente `--retain true` ao `pub` e depois inicie um novo `sub` para o mesmo tópico. O novo assinante verá `retain=true`; os assinantes já ativos recebem a publicação com `retain=false`. Publicar `--retain true` com mensagem vazia remove o valor retido. A ACL precisa conceder o tópico escolhido.

Para testar sessão persistente, use `--clean-session false --client-id offline-lab --qos 2` no `sub`. Quando aparecer “Assinatura ativa”, saia com `Ctrl+C`. Publique QoS 1/2 no mesmo tópico enquanto ele está offline; depois reinicie o broker e execute o `sub` novamente com **o mesmo** `--client-id` e `--clean-session false`. A publicação pendente deve chegar. Use um ID diferente para o `pub`; um segundo cliente com o mesmo ID toma o lugar do primeiro. O diretório `MQTT_STATE_DIR` do broker deve permanecer o mesmo entre reinícios.

Os exemplos acima usam a instância de laboratório `1883`, **com TLS/mTLS mesmo nessa porta**. Configure outros clientes MQTT para TLS; não use conexão sem criptografia. **Antes de executar a nova versão na porta 8883:** a instância antiga do broker precisa ser encerrada e reiniciada com os arquivos de usuários/ACL **e `MQTT_STATE_DIR`**. Recompilar o projeto não atualiza um processo já em execução. Não interrompa uma instância em uso até os arquivos estarem prontos. A prova de integração anterior foi feita em uma instância separada, na porta 18884; ela não alterou o processo em 8883.

Os limites desta CLI são 1024 bytes para tópico/filtro, 4096 bytes para mensagem publicada, 64 KiB por pacote, 8 requisições na fila e timeout de 8 segundos para as etapas iniciais. A sessão usa Clean Session por padrão e Keep Alive de 30 segundos; falhas de conexão encerram a CLI, sem reconexão automática.

## Evidência anterior (cliente somente mTLS)

- `cargo test --locked`: 15 testes aprovados (11 do broker/persistência, 4 do cliente).
- `cargo clippy --all-targets --locked -- -D warnings -A clippy::type_complexity`: aprovado. Sem a exceção, há dois avisos preexistentes de complexidade de tipo em `src/persistence/mod.rs`.
- Cliente `sub --count 1` e cliente `pub` com certificados mTLS trocaram `ola-interop` pelo tópico `teste/mensagem` em Mosquitto 2.0.22 temporário na porta local `18883`. Esse teste antecedeu a exigência de usuário/senha e **não** prova a nova política de autenticação. O contêiner de teste foi removido; o broker do projeto em `8883` não foi alterado.

## Evidência do broker deste projeto

- Com a versão nova do broker em `127.0.0.1:18884`, mTLS, usuário/senha e ACL, `sub --count 1` recebeu `ola-interop` publicado por `pub` em `teste/mensagem` (2026-09-23).
- Senha incorreta recebeu `CONNACK NotAuthorized`; assinatura de `teste/negado` recebeu `SUBACK` de falha.
- O teste encerrou somente a instância temporária na porta 18884. Os arquivos temporários de configuração foram removidos; nenhum usuário de teste foi deixado ativo.

## Evidência da versão 0.2.0

- Na porta de teste `127.0.0.1:18884`, `pub`/`sub` Rust trocaram mensagens em QoS 1 e QoS 2; o publicador recebeu confirmação de cada fluxo.
- Uma publicação QoS 1 retida foi recebida por um novo assinante com `retain=true` e continuou disponível após reiniciar o broker.
- Uma assinatura `CleanSession=0` recebeu uma publicação QoS 2 feita durante sua desconexão, mesmo depois de reiniciar o broker com o mesmo diretório de estado.
- Os testes de crash do WAL e do agregado MQTT passaram. Isso não substitui teste de perda de energia real nem interoperabilidade QoS 1/2 com clientes externos independentes.


## Last Will (0.6 local)

Opcoes --will-topic TOPICO --will-message TEXTO --will-qos 0|1|2
--will-retain true|false. Topic e message sao obrigatorios juntos; QoS padrao0,
retain padraofalse, payload ate4096 bytes e topic ate1024 UTF-8. Mensagem vazia
com retain true remove retained quando Will publica. Ctrl+C/--count e pub
concluido enviam DISCONNECT, cancelando Will; kill/queda publica Will.
