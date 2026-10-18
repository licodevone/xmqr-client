# C138 v1.0 — distribuir bridge MQTT 5 retained QoS 0/1

## Entrada e pré-condição

`main` em `9a02631` após C136/C137; [CI C137](https://github.com/licodevone/xmqr-client/actions/runs/39169197510)
em execução na criação deste prompt. Última prerelease client 0.40.0/
bridge 0.2.0; broker 0.43.0 e admin 0.11.0 publicados. Saída local
`vendor/rumqttc-0.25.1/target/` não rastreada deve ser preservada. Só
promover C138 após CI C137 integral PASS; falhas ficam registradas.

## Objetivo

Publicar prerelease experimental client 0.41.0/bridge 0.3.0 com as duas
novas rotas de origem MQTT 5/QoS 0 e 1, destino MQTT 3.1.1, Retain As
Published e ACK manual QoS 1. Atualizar Cargo/lock, versão da bridge,
Dockerfile, CI, documentação de instalação/rollback, README e VERSIONING.
Corrigir o texto histórico do README que diz que a fase 6 global está
aberta: a fase 6 foi aceita no contrato finito do broker 0.43.0, enquanto
fase 8 segue aberta. Não alterar o comportamento MQTT nesta fatia.

## Aceite

- Gates Rust debug e release, inventário MIT e CI da fonte exata.
- Fixtures fonte MQTT 5 QoS 0/1 com XMQR↔XMQR e XMQR↔Mosquitto, incluindo
  negativos TLS/ACL, replay e retained vivo, crash QoS 1; regressões
  MQTT 3.1.1 QoS 0/1/2 e checkpoint QoS 2.
- `.deb` nativo Ubuntu 26.04 amd64, versões internas corretas e unit opt-in.
  Ciclo descartável 0.40→0.41→0.40→0.41, configuração preservada; testar
  binários extraídos do pacote. Em release sem hook debug, marcar crash
  checkpoint como pulado, com cobertura correspondente na CI debug.
- Imagem não root com labels de versão/revisão, duas versões corretas e
  digest remoto. Tag anotada imutável e prerelease com manifestos,
  SHA-256 e anexos remotos confrontados com os arquivos locais.
- Atualizar o checklist da evolução e o registro C138 somente com
  resultados observados; fase 8 global permanece aberta. Sem `latest`,
  instalação no WSL real do operador ou ativação automática da unit.

## Limites

MQTT 5/QoS 2 na origem, propriedades propagadas, ciclos externos,
transação entre brokers e perda de energia permanecem fora desta release.
No rollback 0.40, a configuração `source_protocol="mqtt5"` deve ser parada
e removida antes de iniciar a bridge antiga; arquivos privados do operador
devem ser preservados.

## Revisão v1.1 — falha de contêiner no ensaio do pacote

O build release, pacote local e ciclo 0.40→0.41→0.40→0.41 passaram. Nos
binários extraídos do `.deb` local, XMQR↔XMQR MQTT 5/QoS 0 e 1, além do
primeiro sentido XMQR→Mosquitto, passaram; `docker run` do segundo sentido
Mosquitto→XMQR encerrou com código 125 duas vezes antes de testar a bridge.
Preservar esses FAILs. Antes de alterar comportamento MQTT, capturar apenas
`stderr` do comando de inicialização do contêiner (sem imprimir senha do
passo `mosquitto_passwd`), distinguir porta ocupada de falha real do broker
e repetir o cenário. Não promover a release até o teste de pacote passar.

## Revisão v1.2 — contexto Docker local

O build Docker local da candidata `eb017e3` falhou duas vezes com
`error from sender: context canceled` enquanto enviava 134 MB do contexto.
O diretório local não rastreado `vendor/rumqttc-0.25.1/target/` ocupa 639 MB;
as regras atuais permitem atravessar a pasta vendorizada e não a excluem
explicitamente. Preservar os dois FAILs e o conteúdo local. Acrescentar
exclusão explícita desse `target/` no `.dockerignore`, validar o tamanho
do contexto e reconstruir a imagem da nova fonte exata antes de publicar.
