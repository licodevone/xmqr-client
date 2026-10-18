# C135 v1.2 — distribuir bridge QoS 2 experimental

## Entrada

`main` em `130cfeb0f99e84264579a4a7a8f05b3d55b8cc4e`, C134 aceito na
fonte pela [CI 39158793553](https://github.com/licodevone/xmqr-client/actions/runs/39158793553).
A CI pós-integração de `main` ainda está em execução na abertura deste prompt.
Última prerelease publicada: client 0.39.0/bridge 0.1.0; broker distribuído
0.42.0. Fases 6 e 8 continuam abertas. Branch deste trabalho:
`release/0.40.0-bridge-qos2`. Não modificar artefatos 0.39.0 já publicados.
A [CI pós-integração 39159131168](https://github.com/licodevone/xmqr-client/actions/runs/39159131168)
da fonte `130cfeb` terminou PASS nos quatro jobs antes do primeiro ajuste de
versão deste prompt.

## Objetivo e escopo

Distribuir C134 como prerelease experimental `xmqr-client 0.40.0`, com
`mqtt-client 0.40.0` e `mqtt-bridge 0.2.0 (xmqr-client 0.40.0)` no `.deb` e na
imagem. Ajustar Cargo/Cargo.lock, Dockerfile, verificações da CI, lifecycle
Ubuntu 26.04 0.39→0.40→0.39, scripts de fixture/versão e documentação. A
bridge QoS 0/1 e o client MQTT 3.1.1/5 preservam comportamento. A unit QoS 2
continua opt-in, com StateDirectory privado. Não instalar no WSL real do
usuário nem iniciar serviço automaticamente.

## Gates de aceite

- Confirmar primeiro a CI pós-integração da base `main`; registrar qualquer
  falha antes de promover a distribuição.
- Rust fmt/test/clippy/build; fixtures QoS 0/1/2 XMQR e Mosquitto, incluindo
  dois checkpoints de crash, perda de Session Present e restart de destino.
- `.deb` Ubuntu 26.04 amd64 da fonte exata, com versões dos dois binários,
  unit instalada mas desabilitada, configuração preservada, upgrade/rollback
  descartável 0.39→0.40→0.39 e retorno à 0.40.
- Testar a bridge da **imagem/pacote release** em QoS 2 com os checkpoints de
  debug explicitamente pulados; a CI debug prova esses checkpoints. Conferir
  imagem não root, labels, versão, digest e revisão da fonte.
- Criar tag anotada e prerelease apenas após CI verde da fonte exata e
  verificações remotas de SHA256 dos seis anexos. Registrar PASS/FAIL/BLOCKED
  e hashes em `prompts/registros/C135-release-0.40.0.md`; atualizar README,
  VERSIONING, nota 0.40.0, documentação de instalação e checklist sem marcar
  fases 6/8 como concluídas.

## Limites

QoS 2 confirma cada perna MQTT, não transação entre brokers nem processamento
do assinante. Retained vivo e ciclos externos continuam fora do aceite T02.
Downgrade para 0.39 preserva arquivos privados sob controle do operador, mas
essa versão não lê o novo journal da bridge; pare instâncias QoS 2 antes de
voltar. Não publicar `latest` sem validação própria.

## Revisão após build local

O `.deb` 0.40.0 foi construído no Ubuntu 26.04, porém o build release emitiu
avisos para `Outgoing` e `pending_topic`, usados apenas nos ganchos de crash do
build debug. Aplicar `cfg(debug_assertions)` nesses dois pontos sem alterar o
runtime release e repetir Clippy release/build/pacote antes do aceite. O aviso
`dpkg-shlibdeps` sobre diversions da própria libc6 é separado e não foi
interpretado como erro do pacote.
