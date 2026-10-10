# Ubuntu 26.04 / WSL: client e systemd

Alvo: Ubuntu26.04 amd64. Construção requer Rust, Python3 e dpkg-dev:

```sh
python3 scripts/build_deb.py
sudo apt install ./dist/xmqr-client_0.5.0-1_amd64.deb
mqtt-client --version
```

O pacote instala o binário em `/usr/bin`, licenças e duas units template
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
