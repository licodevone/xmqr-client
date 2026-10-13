# xmqr-client

C45 acrescenta prazo total monotônico opcional e jitter limitado de reconexão,
separa CONNACK de reconexão do idle e prioriza cancelamentoSIGINT/SIGTERM.
Veja [robustez da sessão](docs/client-robustness.md) e [registroC45](prompts/registros/C45-robustez-reconexao.md).
O marco de release será registrado somente após os gates coordenados.

O client participa do [checklist da fase2](docs/phase2-acceptance.md).
C43 amplia interop externa sem mudar runtime0.8.1; P85/C44 completaram
o aceite físico ESP32 e encerraram a fase2 no escopo validado, sem declarar produção.

Marco de distribuição atual: **0.8.1**, com imagem
`lepinheiro100/xmqr-client:0.8.1`, .deb Ubuntu26 amd64 e manifesto/checksums.
Mantém o comportamento da 0.8.0. O [plano detalhado de paridade](https://github.com/licodevone/xmqr/blob/main/docs/plano-paridade-mosquitto.md)
está concluído como planejamento, sem implementar fases futuras nesta entrega.

Projeto Rust independente, extraído do cliente XMQR. Pacote `xmqr-client`,
binário compatível `mqtt-client`. A versão **v0.8.0** acrescenta filtros retained
na saída do assinante. O marco **v0.7.0** concluiu a fase1 com timeout, keepalive e limites;
inclui integração systemd e pacote Ubuntu26.04 amd64. Veja [instalação e serviços](docs/ubuntu-systemd.md).
Projeto experimental; consulte as prereleases no GitHub.

## Instalação Ubuntu / WSL

O pacote experimental abaixo é destinado ao Ubuntu 26.04 **amd64**. Ainda não
há repositório APT próprio: baixe o `.deb` e instale-o com o APT.

1. No Windows, abra o Ubuntu pelo PowerShell. Confira o nome da distribuição
   com `wsl --list --verbose` e ajuste o comando se necessário:

```powershell
wsl -d Ubuntu-26.04
```

2. Execute os próximos comandos **no terminal do Ubuntu**. Confirme Ubuntu
   26.04 e arquitetura `amd64`, depois prepare o download:

```bash
cat /etc/os-release
dpkg --print-architecture
sudo apt update
sudo apt install -y wget ca-certificates
mkdir -p ~/xmqr-pacotes
cd ~/xmqr-pacotes
```

3. Baixe o pacote e confira sua integridade:

```bash
wget -O xmqr-client_0.8.1-1_amd64.deb https://github.com/licodevone/xmqr-client/releases/download/v0.8.1/xmqr-client_0.8.1-1_amd64.deb
wget -O xmqr-client_0.8.1-1_amd64.deb.sha256 https://github.com/licodevone/xmqr-client/releases/download/v0.8.1/xmqr-client_0.8.1-1_amd64.deb.sha256
sha256sum -c xmqr-client_0.8.1-1_amd64.deb.sha256
```

Prossiga somente se a verificação mostrar `OK`.

4. Instale e confirme o pacote, o executável e a conta de serviço:

```bash
sudo apt install ./xmqr-client_0.8.1-1_amd64.deb
dpkg-query -W -f='${Status}\n' xmqr-client
mqtt-client --version
id xmqr-client
```

O resultado esperado da consulta é `install ok installed`. O pacote cria
automaticamente o usuário e grupo `xmqr-client`. O executável chama-se
`mqtt-client`; `mqtt-admin` é instalado pelo pacote do broker.

5. Para conectar com TLS/mTLS e senha, configure o broker primeiro e siga o
   [guia de certificados, credenciais e serviços](docs/ubuntu-systemd.md).
   Para a instância `device`, prepare `/etc/xmqr-client/device.conf` e os
   arquivos `ca.crt`, `client.crt`, `client.key` e `password` em
   `/etc/xmqr-client/device/`. A chave e o arquivo de senha devem pertencer a
   `xmqr-client:xmqr-client`, com permissão `0600`. O usuário MQTT e as ACLs
   precisam existir no broker, e o endereço usado deve corresponder ao SAN do
   certificado do servidor. Não coloque senhas na linha de comando ou no Git.

6. Para manter a assinatura como serviço, após configurar a instância:

```bash
sudo systemctl enable --now xmqr-client-sub@device.service
systemctl status xmqr-client-sub@device.service
journalctl -u xmqr-client-sub@device.service -f
```

Use `Ctrl+C` para sair dos logs. O pacote não inicia instâncias automaticamente.
No WSL, habilite o systemd conforme o guia; a instância habilitada inicia quando
a distribuição é iniciada. Para uso manual, consulte `mqtt-client --help` e os
exemplos do guia.

Para atualizar, baixe e verifique o novo pacote e repita
`sudo apt install ./ARQUIVO.deb`. A configuração é preservada, mas os serviços
são parados; revise os arquivos e inicie novamente as instâncias desejadas.

## Compatibilidade Ubuntu

| Ambiente amd64 | Evidência atual |
| --- | --- |
| Ubuntu 26.04 / WSL2 | Pacotes instalados e testes de integração e systemd com TLS/mTLS aprovados. Alvo atual dos pacotes publicados. |
| Ubuntu 24.04 / WSL2 | Os mesmos binários executaram testes de integração; instalação do `.deb` e units nessa versão ainda não validadas. |
| Outras versões Ubuntu, outras distribuições ou arm64 | Não validadas. Não há pacote arm64 publicado nesta etapa. |

Os pacotes requerem `libc6 >= 2.38`, `libgcc-s1 >= 4.2`, `adduser` e
`init-system-helpers`. O Ubuntu 24.04 testado tem glibc 2.39 e executa os
binários. O teste de sessão após reinício recebeu uma mensagem QoS1 repetida na
primeira execução e passou na repetição; a causa ainda precisa ser confirmada.
Não considere essa validação parcial como suporte completo ao 24.04. O gerador
de pacotes permanece restrito ao Ubuntu 26.04 amd64. Não force dependências nem
substitua a glibc do sistema para instalar um pacote.

## Controlar os clients com systemctl

Execute os comandos no terminal Ubuntu/WSL. Configure o broker, certificados,
senha e ACLs antes de iniciar. O nome após `@` identifica a instância: `device`
usa `/etc/xmqr-client/device.conf` e os arquivos em `/etc/xmqr-client/device/`.
Substitua `device` pelo nome de sua instância em todos os comandos.

### Tabela comparativa de comandos

Execute no Ubuntu/WSL. `device` é o nome da instância do client de assinatura;
substitua-o pelo nome configurado em `/etc/xmqr-client/NOME.conf`.

| Ação | Broker | Client de assinatura |
| --- | --- | --- |
| Iniciar | `sudo systemctl start xmqr-broker.service` | `sudo systemctl start xmqr-client-sub@device.service` |
| Ver status | `systemctl status xmqr-broker.service` | `systemctl status xmqr-client-sub@device.service` |
| Parar | `sudo systemctl stop xmqr-broker.service` | `sudo systemctl stop xmqr-client-sub@device.service` |
| Reiniciar | `sudo systemctl restart xmqr-broker.service` | `sudo systemctl restart xmqr-client-sub@device.service` |
| Iniciar agora e automaticamente | `sudo systemctl enable --now xmqr-broker.service` | `sudo systemctl enable --now xmqr-client-sub@device.service` |
| Parar e desativar início automático | `sudo systemctl disable --now xmqr-broker.service` | `sudo systemctl disable --now xmqr-client-sub@device.service` |
| Verificar se está ativo | `systemctl is-active xmqr-broker.service` | `systemctl is-active xmqr-client-sub@device.service` |
| Verificar início automático | `systemctl is-enabled xmqr-broker.service` | `systemctl is-enabled xmqr-client-sub@device.service` |

### Client de assinatura: processo contínuo

| Objetivo | Comando | Efeito |
| --- | --- | --- |
| Iniciar | `sudo systemctl start xmqr-client-sub@device.service` | Inicia a assinatura agora. |
| Consultar status | `systemctl status xmqr-client-sub@device.service` | Mostra estado, processo e registros recentes. |
| Parar | `sudo systemctl stop xmqr-client-sub@device.service` | Solicita encerramento do client. |
| Reiniciar | `sudo systemctl restart xmqr-client-sub@device.service` | Para e inicia novamente, aplicando a configuração. |
| Habilitar início automático | `sudo systemctl enable xmqr-client-sub@device.service` | Habilita para a próxima inicialização, sem iniciar agora. |
| Habilitar e iniciar agora | `sudo systemctl enable --now xmqr-client-sub@device.service` | Combina início automático e início imediato. |
| Desabilitar início automático | `sudo systemctl disable xmqr-client-sub@device.service` | Remove o início automático, sem parar agora. |
| Desabilitar e parar | `sudo systemctl disable --now xmqr-client-sub@device.service` | Remove o início automático e para agora. |
| Consultar atividade | `systemctl is-active xmqr-client-sub@device.service` | Exibe `active`, `inactive` ou `failed`, por exemplo. |
| Consultar início automático | `systemctl is-enabled xmqr-client-sub@device.service` | Exibe `enabled` ou `disabled`, por exemplo. |

`active (running)` indica processo em execução; confira os logs e a entrega de
uma mensagem para confirmar a assinatura. `inactive (dead)` significa parado;
`failed` indica uma falha. Se a configuração da instância estiver ausente, sua
condição de início pode impedir a execução: confira o status e o caminho do
arquivo. Para `Unit ... could not be found`, verifique a instalação com
`dpkg-query -W xmqr-client`.

Instâncias simultâneas precisam de ClientIDs diferentes. Uma assinatura
contínua pode ser mantida enquanto outra instância publica. Consulte o
[controle do broker](../xmqr/README.md#controlar-o-broker-com-systemctl) para
iniciar o servidor primeiro.

### Client de publicação: execução pontual

A unit `xmqr-client-pub@device.service` publica o conteúdo de
`/etc/xmqr-client/device/payload.bin`, com limite de 4096 bytes. Prepare o arquivo
para que o usuário `xmqr-client` consiga lê-lo e configure `TOPIC` como tópico
de publicação, sem os curingas `+` ou `#`. Para manter publicação e assinatura
simultâneas, prefira instâncias distintas, como `publisher` e `subscriber`, com
seus próprios arquivos `.conf`, diretórios e ClientIDs.

```bash
# Executar uma publicação
sudo systemctl start xmqr-client-pub@device.service

# Conferir estado e resultado
systemctl status xmqr-client-pub@device.service --no-pager
journalctl -u xmqr-client-pub@device.service -n 50 --no-pager

# Interromper uma publicação que ainda esteja em execução
sudo systemctl stop xmqr-client-pub@device.service
```

Esse serviço é `oneshot`: termina após a tentativa de publicação e não reinicia
automaticamente. Após sucesso pode aparecer como `inactive (dead)`; confira
nos logs o evento `publish_complete` e seu resultado de protocolo. Para
consultar o código de saída do processo:

```bash
systemctl show xmqr-client-pub@device.service --property=ExecMainStatus --value
```

O código `0` indica saída com sucesso, mas deve ser interpretado junto com os
logs e a confirmação prevista pelo QoS. Não comprova processamento pelo
destinatário. Repetir `start` ou usar `restart` pode publicar a mensagem de novo.
`stop` não desfaz uma mensagem já enviada. A unit de publicação não possui
seção de instalação para `enable` nem timer; use `start` para cada envio.

### Logs dos clients

```bash
# Acompanhar a assinatura em tempo real
journalctl -u xmqr-client-sub@device.service -f

# Consultar as últimas 50 linhas
journalctl -u xmqr-client-sub@device.service -n 50 --no-pager

# Consultar registros desta inicialização do Ubuntu
journalctl -u xmqr-client-sub@device.service -b --no-pager

# Consultar registros recentes da publicação
journalctl -u xmqr-client-pub@device.service --since "10 minutes ago" --no-pager

# Listar as units de clients atualmente carregadas
systemctl list-units --all 'xmqr-client-*'
```

`journalctl` consulta os registros do systemd; `-u` filtra a unit e `-f` mantém
o acompanhamento de novos registros. Não inicia nem para o client. Pressione
**Ctrl+C** para sair dos logs e **q** para sair da tela paginada de status.
Use `--no-pager` para consultar status sem essa tela. Se o acesso aos logs for
negado, execute a consulta com `sudo`.

### Configuração, falhas e WSL

Depois de alterar o `.conf`, senha ou certificados, reinicie a assinatura com
`sudo systemctl restart xmqr-client-sub@device.service`. Os clients não possuem
`ExecReload`: não use `systemctl reload` para aplicar essas alterações. A
publicação lê os arquivos no próximo `start`.

Se editar uma unit ou override, execute `sudo systemctl daemon-reload` e depois
reinicie a instância desejada. `daemon-reload` recarrega as definições do
systemd, sem iniciar ou reiniciar os clients sozinho.

A assinatura pode reiniciar após falhas, com limite de cinco inícios por 120
segundos. Erros de configuração com saída `2` não provocam reinício automático.
Após corrigir o problema, consulte os logs e, se o limite de início foi atingido:

```bash
sudo systemctl reset-failed xmqr-client-sub@device.service
sudo systemctl start xmqr-client-sub@device.service
```

`reset-failed` limpa o estado de falha, sem corrigir a configuração. Uma parada
solicitada por `stop` não provoca reinício automático. Antes de remover o
pacote, desabilite as instâncias de assinatura com `disable --now`.

No WSL, `ps -p 1 -o comm=` deve mostrar `systemd`; consulte o
[guia Ubuntu e serviços](docs/ubuntu-systemd.md) para habilitá-lo. Instâncias
habilitadas iniciam quando a distribuição Ubuntu é iniciada. Os comandos
controlam os serviços dentro dessa distribuição.

## Diagnóstico e confirmação de telemetria — C35

O client diferencia recusas MQTT de autenticação, autorização, versão e ClientID.
Falhas TLS indicam revisão de CA/SAN/validade e certificado/chave; outras falhas
de transporte não são classificadas automaticamente como erro de certificado.
Mensagens não incluem conteúdo recebido, senhas ou detalhes privados de erros.
O comportamento de reconexão permanece limitado; autenticação recusada não
provoca novas tentativas automáticas.

Para confirmar a telemetria do ESP32, use um usuário com ACL de assinatura para
`camara/cold-room-sim-01/telemetria` e um ClientID diferente do dispositivo.
No Ubuntu, o comando abaixo solicita a senha sem exibi-la. Substitua
`USUARIO_AUTORIZADO` pelo usuário cadastrado e confira os caminhos dos arquivos:

```bash
mqtt-client sub \
  --host 127.0.0.1 --port 8883 \
  --topic camara/cold-room-sim-01/telemetria \
  --username USUARIO_AUTORIZADO --client-id telemetry-check \
  --qos 1 --count 5 --output jsonl \
  --ca ~/xmqr-certificados/ca.crt \
  --cert ~/xmqr-certificados/client.crt \
  --key ~/xmqr-certificados/client.key
```

O exemplo é para execução manual; não registra automaticamente dados reais.
Cinco eventos recebidos comprovam entrega ao client, não processamento de negócio.
QoS1 permite duplicatas; para contagem lógica, compare o identificador/sequência
do payload da aplicação. A assinatura QoS1 não aumenta o QoS da publicação.
Não usar o mesmo ClientID do ESP32: isso pode desconectar a conexão existente.

## Compilar a partir do código

```powershell
cd D:\projects\my-project\rapidez-projects/rapidez-broker-client\xmqr-client
cargo build --locked --bin mqtt-client
cargo run --locked -- --help
```

No WSL use `/mnt/d/projects/my-project/rapidez-projects/rapidez-broker-client/xmqr-client`. O broker
independente permanece em `D:/projects/my-project/rapidez-projects/rapidez-broker-client/xmqr`;
`mqtt-admin` pertence ao broker.
Veja [a imagem Docker do cliente](docs/docker.md) para build e montagem de
certificados.
Cliente usa rumqttc: pub/sub QoS0/1/2, retained, sessões e Last Will; TLS/mTLS
com validação de CA/SAN e credenciais. Laboratórios somente loopback.
Tópicos/filtros: até 1024 bytes UTF-8; payload até 4096. Melhorias C26 abaixo.
Senha interativa oculta; `--password-file` somente Linux/WSL, proprietário
atual e modo 0600. Não há persistência local de sessão. Reconexão opcional descrita abaixo.

Gates: `cargo fmt --all -- --check`, `cargo test --locked`,
`cargo clippy --locked --all-targets -- -D warnings`, `cargo build --locked`.
Integração: no broker, execute `scripts/verify_last_will.py --broker CAMINHO
--client CAMINHO_DO_CLIENTE_INDEPENDENTE` em ambiente Unix isolado.
Guia anterior preservado em docs/original-mqtt-client.md como referência:
caminhos de build e links relativos nele pertencem ao repositório original.
Prompts C19-C24 são históricos/propostas, não autorização de implementação.

Toda melhoria deve começar por um prompt com ID/versão, estado, objetivo,
escopo, arquivos, invariantes, aceite e testes. Veja AGENTS.md e C25.
MIT e atribuições originais preservadas. Separação estrutural concluída com gates Windows e backup verificado.
Validações Unix com broker isolado e peer TLS próprio estão no registro C27;
MSRV1.88 exato e interoperabilidade externa ainda não comprovados.


## Marco C26 (incluído na tag v0.2.0)

A etapa C26 preservou o manifest herdado 0.7.0. C28 inicia a numeração
independente 0.3.0, sem reescrever a tag anterior.

`--reconnect-attempts N` aceita 0..10 (padrão 0, falha sem reconectar).
Usa o mesmo EventLoop rumqttc, Client ID e opções de sessão. O orçamento é total
por processo, incluindo falhas antes do CONNACK e quedas após conectar, sem reset.
Desde C45, as bases são 100, 200, 400, 800, 1600, 3200ms e 5s nas demais;
cada espera usa jitter limitado80..100% da base, sem ultrapassar5s;
cada conexão mantém os timeouts da biblioteca e os deadlines do cliente.
TLS, recusa de autenticação e erros de protocolo não são retentados.

Em `sub`, sessão retomada preserva a assinatura; sessão ausente exige SUBSCRIBE
novamente. Use --client-id ID --clean-session false para solicitar sessão estável
entre conexões/processos; a retomada depende de o broker ainda ter essa sessão.
Não há persistência de estado MQTT no disco do cliente. --count continua contando
através das reconexões. Ctrl+C cancela poll/espera e tenta DISCONNECT por até 1s
se o transporte ainda estiver disponível, sem reconectar durante o encerramento.

Em `pub`, reconexão é permitida somente antes de enfileirar PUBLISH. Se a conexão
cair enquanto se aguarda o envio/ACK, o cliente falha com resultado desconhecido,
sem reenviar a publicação. Uma repetição manual pode produzir duplicatas.
QoS0 indica envio; PUBACK/PUBCOMP confirmam apenas o protocolo do broker,
não sucesso de negócio nem processamento pelo assinante.

`--output text|jsonl` mantém texto por padrão. JSON Lines escreve um objeto por
linha em stdout; estados operacionais, reconexões e erros vão para stderr.
Mensagens têm event, topic, qos, retain e payload_bytes (array de
inteiros 0..255, inclusive payload vazio), preservando binários sem conversão UTF-8.
Conclusão de pub usa event: "publish_complete" e confirmation igual a sent
(QoS0) ou broker_protocol_ack (QoS1/2). Credenciais não integram esses objetos.

```powershell
cargo run --locked -- sub --open-lab --topic 'test/#' --count 2 --client-id stable-id --clean-session false --reconnect-attempts 3 --output jsonl
```

Validação C26 usa fixtures MQTT próprias em loopback/portas efêmeras, sem executar
o broker vizinho ou Mosquitto. Isso não substitui os gates de integração Unix/TLS
com broker independente; a execução isolada posterior está registrada em C27.

Limitação verificada: rumqttc 0.25.1 limpa o estado de recepção QoS2 ao perder
a conexão. Se a queda ocorrer entre PUBLISH recebido e conclusão PUBREL/PUBCOMP,
o cliente falha explicitamente, sem reconectar nessa fase. O handshake recebido
tem deadline padrão de 8s, configurável por `--timeout-seconds`; `--count` aguarda sua conclusão antes de DISCONNECT.

Prompts: veja [índice e ordem](prompts/README.md) e [matriz de cobertura](prompts/COBERTURA.md).
Os históricos não constituem um bootstrap independente; use o contrato C27.

## Checkpoint C27

[Contrato de bootstrap](prompts/clients/27-bootstrap-independente.md) cobre o cliente
até C26. [Reconstrução de validação](validation/c27-bootstrap/README.md) tem fontes
novos, insumos reutilizados explicitamente e comparação por contratos, sem substituir
este cliente ou prometer identidade de bits. [Resultados e limitações](prompts/registros/C27-bootstrap.md).

O binário cliente é Rust; Python auxilia testes, inventário e conferência de hashes,
não é um cliente alternativo nem requisito para executar mqtt-client. TOML/JSON são
configurações/metadados; Markdown é documentação. Comandos usam PowerShell/bash.

## Marco C28 — incluído na tag v0.3.0

[Prompt C28 rev.2.0](prompts/clients/28-payload-binario-planejado.md) e
[registro de validação](prompts/registros/C28-payload-binario.md).
Use exatamente uma fonte em pub: --message TEXTO, --message-file CAMINHO ou
--stdin. Arquivo deve ser regular; erros não exibem caminhos ou bytes recebidos.
O modo bruto preserva todos os bytes (BOM, NUL, UTF-8 inválido e CRLF), incluindo
payload vazio, até 4096 bytes; aguarda EOF. Não converte a entrada em texto.

--line-mode, somente com arquivo/stdin, publica uma mensagem por linha: remove
LF e um CR imediatamente anterior, preserva CR isolado, BOM e bytes binários.
Linha vazia publica payload vazio; EOF publica última linha sem LF; entrada
vazia não conecta. LF final não cria mensagem extra. O limite de4096 bytes vale
por payload após esse enquadramento. Retain vale para cada linha: uma linha
vazia com --retain true apaga o retained desse tópico.

Cada publicação aguarda envio QoS0 ou ACK terminal QoS1/2 antes da seguinte.
No modo por linha, cada conclusão escreve o mesmo formato text/JSONL de C26.
Uma falha posterior retorna erro e mantém as conclusões anteriores; não há
rollback. Depois do primeiro PUBLISH não reconecta nem reenvia o lote.
ACK confirma protocolo, não processamento pelo assinante ou sucesso de negócio.

A primeira entrada é validada antes de credenciais/rede. Cada espera de entrada
tem limite fixo de8s, sem renovação por eventos MQTT. Ctrl+C cancela a espera e
tenta DISCONNECT no link ativo por até1s. A leitura nativa bloqueada não impede
o encerramento do processo; fila limitada e publicação serial aplicam backpressure.
Senha oculta usa console/TTY separado; --password-file preserva proteção Linux0600.

~~~powershell
cargo run --locked -- --version
cargo run --locked -- pub --open-lab --topic 'test/binary' --message-file 'payload.bin' --qos 1 --output jsonl
cargo run --locked -- pub --open-lab --topic 'test/lines' --message-file 'lines.bin' --line-mode --qos 2
~~~

Para stdin binário use um pipe que preserve bytes; comandos de texto do shell
podem recodificar a entrada antes de o cliente recebê-la. --version imprime
a versão atual sem ler entrada/senha ou abrir conexão.

Gates Windows/Linux e18 cenários isolados (C28+C27+Will) passaram; MSRV1.88
exato não instalado e Mosquitto/broker TLS completo continuam não comprovados.
O bootstrap C27 é evidência congelada do contrato da tag0.2; não é reconstrução
independente da nova versão0.3. Não executar seu checker antigo para declarar
igualdade com os fontes após C28.

## Encerramento de serviço

SIGTERM/SIGINT usam DISCONNECT limitado a1s no link disponível. A unit de
publicação não reinicia; a de assinatura tem reinícios limitados. Atualizações
param instâncias; reinicie explicitamente após revisar configuração.

## Filtros retained na assinatura — C41

`sub --ignore-retained` omite mensagens recebidas com RETAIN=true;
`sub --retained-only` exibe somente essas mensagens. Sem flag, todas são exibidas.
As opções são exclusivas, não aceitam valor e não valem para `pub`.

```sh
mqtt-client sub --open-lab --topic 'sensor/#' --ignore-retained --count 1 --output jsonl
mqtt-client sub --open-lab --topic 'sensor/#' --retained-only --count 1 --output jsonl
```

`--count` conta somente mensagens exibidas. Mensagens filtradas continuam sendo
recebidas e confirmadas em QoS1/2; o filtro não muda a assinatura nem a ACL.
RETAIN é a flag recebida do broker: o replay de retained normalmente a usa,
enquanto publicações ao vivo chegam com RETAIN=false mesmo quando o publicador
usou `--retain true`. O filtro não deduz quais tópicos têm retained armazenado.
Text e JSONL seguem a mesma regra; reconexão, cancelamento e timeouts permanecem.

## Publicação verificada

Prerelease da fase1: [v0.7.0](https://github.com/licodevone/xmqr-client/releases/tag/v0.7.0),
com deb Ubuntu26 amd64 e checksum. Aceite em
[registro C39](prompts/registros/C39-release-fase1.md).


## Evolução e paridade

Veja a [matriz Mosquitto](docs/mosquitto-parity.md), com lacunas, evidências e
próximos incrementos. C36 é documental; C37 implementa timeout/keepalive com fixtures locais.

## Timeout e keepalive — C37

`--keepalive-seconds N` aceita 1..65535, padrão 30, enviado em CONNECT e
preservado na reconexão. Zero, embora permitido pelo MQTT, fica fora deste contrato.
`--timeout-seconds N` aceita 1..300, padrão 8. Limita cada poll/tentativa antes
do CONNACK e os deadlines de SUBACK, envio/ACK terminal de PUBLISH, conclusão
do QoS2 recebido e DISCONNECT normal. Não limita o comando inteiro, espera por
mensagens de aplicação ou backoff de reconexão. O orçamento de retry permanece
inalterado; expirar o deadline externo não cria nova tentativa.

A assinatura mantém sua inatividade de 90s, entrada de payload mantém 8s e
cancelamento mantém DISCONNECT de até 1s. Publicação interrompida tem resultado
desconhecido e não é reenviada. A opção não altera os timeouts internos do rumqttc.

C45 corrige a renovação do idle: após SUBACK, somente PUBLISH recebido reinicia
90s; PINGRESP/eventos de saída não mascaram ausência de mensagens. Na reconexão,
CONNACK usa protocolo configurado, não idle90s. SUBACK/idle incluem backoff.

`--total-timeout-seconds N` aceita1..86400, opcional; padrão sem prazo total.
Usa uma deadline monotônica desde session::run após a primeira entrada de payload
e senha, incluindo TLS/rede/backoff/protocolo/mensagens/entrada posterior e
DISCONNECT normal. Nunca renova; não limita a senha interativa nem a entrada
inicial (8s). Expiração sai com erro, sem reenvio nem período adicional.
SIGINT/SIGTERM já pendentes são priorizados sobre poll e nova tentativa.

Exemplo local: `mqtt-client sub --open-lab --topic test/# --keepalive-seconds 45 --timeout-seconds 3`.
Veja [registro C37](prompts/registros/C37-timeout-keepalive.md).

## Limites da fase1 — C38

Payload de publicação até4096 bytes; tópico/filtro até1024 bytes UTF-8. O SUBSCRIBE de saída inclui cabeçalho e corpo no teto65536 bytes:65536 aceito,65537 recusado antes de rede.

Na recepção, rumqttc0.25.1 limita Remaining Length, não o frame completo. O client configura65531 bytes de corpo e reserva o cabeçalho máximo de5bytes; assim nenhum frame aceito excede65536 bytes, inclusive comprimento codificado sem a forma mínima. Essa proteção pode recusar um frame canônico de65536 bytes por1byte. Não afeta as publicações suportadas pelo broker (payload4096/tópico1024). Não há promessa de fronteira inclusiva idêntica de entrada/saída; uma necessidade externa dessa fronteira exige escopo próprio para o codec.

SUBACK recusado orienta conferir ACL/limites; MQTT3.1.1 não informa causa específica. Uma queda de transporte não comprova quota, e publicação interrompida mantém resultado desconhecido sem reenvio. PUBACK/PUBCOMP confirmam protocolo, sem reason de quota.

Integração real isolada (Unix, sem usar serviços/estado instalado):

```bash
python3 scripts/verify_phase1.py --broker ../xmqr/target/debug/mqtt-broker --client target/debug/mqtt-client --report /tmp/c38-phase1.json
```

O harness usa somente loopback e diretórios temporários; verifica4096/4097 QoS0/1/2, keepalive1/timeout1, SUBSCRIBE65536/65537 e recusa por limite de assinaturas. Confira o [registroC38](prompts/registros/C38-limites-fase1.md): preparação ou fixtures não equivalem ao aceite Unix.

## Aceite final da fase1 — C39

Ubuntu26/WSL:52 testes, fmt/Clippy/build e integração com broker0.11.0 aprovados.
Instalação de0.5.0, upgrade para0.7.0, rollback, remove/purge e preservação de
arquivos foram testados em container descartável. Ver [registroC39](prompts/registros/C39-release-fase1.md).
Históricos C37/C38 com BLOCKED foram superados nesse ambiente; não são novos
resultados Windows, Ubuntu24, arm64 ou MSRV1.88 exato.

## Docker — distribuição P63/C40

Imagem lepinheiro100/xmqr-client:0.8.1 para Linux/amd64 com tar.gz/SHA-256 e manifesto na prerelease; também inclui
.debUbuntu26amd64. [Build/load, configuração e execução](docs/docker.md).
Sem chaves/estado na imagem; usuário10001, mTLS e volumes externos.
