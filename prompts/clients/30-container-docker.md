# C30 — imagem Docker do cliente

Prompt 1.0, 2026-10-10. Estado: preparado antes dos arquivos Docker. Pedido
autorizado: criar e validar uma imagem Docker local independente. Base própria
do cliente: candidato 0.4.0 após C29.

## Estado de entrada

Repositório independente `xmqr-client`, pacote Rust `xmqr-client`, binário
`mqtt-client`, Rust/MIT. A versão atual é candidato local 0.4.0; `pub` e `sub`
recebem opções do usuário pela linha de comando. O cliente não incorpora broker.

## Escopo

Adicionar Dockerfile multi-stage e `.dockerignore` ao repositório, imagem Debian
slim, usuário sem privilégios e `ENTRYPOINT` do binário `mqtt-client`. Manter
argumentos e entrada padrão disponíveis, incluindo publicação por stdin/arquivo,
modo por linha, JSON Lines em stdout e diagnósticos em stderr. Montar CA/cert/key
como arquivos externos; nunca copiá-los ou credenciais para a imagem. Documentar
build, `--version`, pub/sub, volumes somente leitura e limites de rede.

O modo aberto segue preso a loopback dentro de cada container; esclarecer que
`--open-lab` não conecta a um broker em outro container. A configuração segura
TLS/mTLS é o caminho de exemplo para rede entre containers.

## Aceite

- `docker build` conclui com `Cargo.lock`; runtime não contém Rust/Cargo/source.
- `docker run --rm IMAGE --version` informa 0.4.0 e o entrypoint aceita subcomando.
- `docker run -i` encaminha stdin binário e `--line-mode`; stdout segue JSONL e
  logs permanecem em stderr.
- A integração de imagem deve ficar em `validation/C30-docker.py`, receber as
  imagens por argumentos e limpar containers/volumes de teste no `finally`.
- Usuário runtime não-root; certificados e senha não estão na imagem/logs.
- Registrar plataforma, comandos de build e smoke tests. Sem `docker push`, sem
  tag/release Git, sem upgrade de dependência e sem instalar Mosquitto.

A tag `v0.4.0` continua a cargo do usuário.
