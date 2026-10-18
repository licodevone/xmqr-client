# Ubuntu 26.04 / WSL: client e systemd

Alvo: Ubuntu26.04 amd64. Construção requer Rust, Python3 e dpkg-dev:

```sh
python3 scripts/build_deb.py
sudo apt install ./dist/xmqr-client_0.37.0-1_amd64.deb
mqtt-client --version
mqtt-bridge --version
```

O pacote instala `mqtt-client` e `mqtt-bridge` em `/usr/bin`, licenças e três units template
opcionais. Não ativa nem inicia clientes. Cria usuário sem login `xmqr-client`.
Copie `/usr/share/doc/xmqr-client/client.conf.example` para
`/etc/xmqr-client/device.conf` e configure host, tópico, usuário, QoS e ClientID.
Crie `/etc/xmqr-client/device/` com ca.crt, client.crt, client.key e password.
Senha/chave privadas pertencem ao usuário xmqr-client, modo0600; diretório
legível por esse usuário e inacessível a outros. Certificado deve ter SAN
compatível com o host. Não coloque senhas no arquivo de ambiente ou argv.

```sh
sudo systemctl enable --now xmqr-client-sub@device.service
systemctl status xmqr-client-sub@device.service
journalctl -u xmqr-client-sub@device.service -f
sudo systemctl stop xmqr-client-sub@device.service
```

Sub mantém sessão solicitada com clean-session=false e reconexão limitada.
SIGTERM e SIGINT tentam DISCONNECT por até1s no link disponível, sem reconectar
no encerramento. Isso cancela Will apenas se o broker receber DISCONNECT.
Há no máximo5 inícios por120s; erros de configuração (exit2) não reiniciam.
Estado MQTT local não é persistido; retomada QoS2 interrompida continua limitada.
Use ClientIDs diferentes para instâncias simultâneas.

Para publicação pontual, forneça também `device/payload.bin` (até4096 bytes):

```sh
sudo systemctl start xmqr-client-pub@device.service
journalctl -u xmqr-client-pub@device.service
```

Essa unit oneshot não reinicia. Repetir start pode duplicar uma publicação;
ACK só comprova protocolo. Não há timer instalado nem reenvio automático.
As templates usam um filtro e TLS/mTLS; múltiplos filtros e outros parâmetros
podem ser configurados por um override de ExecStart explícito.

O sidecar experimental C125–C127 fica disponível como `mqtt-bridge`, mas não
é iniciado pela instalação. Para uma instância `lab`, crie
`/etc/xmqr-client/bridge-lab.toml` e o manifesto de topologia em
`/etc/xmqr-client/bridge-topology.toml` conforme o
[guia](bridge-one-way.md). Ambos devem pertencer a `xmqr-client:xmqr-client`
com modo `0600`; certificados CA/públicos podem ter leitura mais ampla, mas
senhas e chaves privadas exigem `0600` e acesso pelo mesmo usuário. Use
caminhos absolutos dentro de `/etc/xmqr-client` no TOML. A unit não contém
segredos nem lê variáveis de senha.

```sh
sudo chown xmqr-client:xmqr-client /etc/xmqr-client/bridge-lab.toml /etc/xmqr-client/bridge-topology.toml
sudo chmod 0600 /etc/xmqr-client/bridge-lab.toml /etc/xmqr-client/bridge-topology.toml
sudo systemctl enable --now xmqr-client-bridge@lab.service
systemctl status xmqr-client-bridge@lab.service --no-pager
journalctl -u xmqr-client-bridge@lab.service -n 50 --no-pager
```

Use um ClientID de origem estável e exclusivo por bridge QoS 1. A topologia
deve ser idêntica entre as instâncias gerenciadas, reiniciadas após alteração.
O pacote 0.36.0 anterior não inclui `mqtt-bridge` nem essa unit; em rollback,
desabilite a instância antes de instalar 0.36.0. O pacote preserva os arquivos
de configuração do operador na remoção.

Upgrade/remove param as instâncias e preservam configurações externas. Após
upgrade, reinicie as instâncias desejadas manualmente. Remove/purge não apagam
credenciais do operador nem usuário. Antes de remoção definitiva, desabilite
instâncias habilitadas com systemctl disable --now para retirar links locais.

No WSL, units habilitadas iniciam quando a distribuição é iniciada. Build e
deb são Ubuntu26 amd64; não se presume Ubuntu24, arm64 ou MSRV1.88 exato.
Distribuição por GitHub prerelease com deb/checksum; ainda não existe
repositório APT próprio assinado. Instalar arquivo local via apt é suportado.

[systemd.service](https://github.com/systemd/systemd/blob/main/man/systemd.service.xml)
explica templates, oneshot e políticas de reinício.
