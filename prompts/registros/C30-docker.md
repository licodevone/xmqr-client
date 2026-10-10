# Registro C30 — imagem Docker do cliente

Data: 2026-10-10. Imagem local `xmqr-client:0.4.0`; nenhum push de imagem,
tag Git ou release. Docker Desktop 4.93.0, Docker Engine 29.8.1,
Linux/amd64 (`desktop-linux`).

## Resultado

Dockerfile multi-stage com builder Rust 1.99.0 e runtime Debian slim. Contém
somente o binário `mqtt-client`, certificados públicos do sistema, licença MIT
e inventário das dependências. Roda como UID/GID 10001 e encaminha argumentos,
stdin/stdout e stderr pelo entrypoint JSON array.

## Validação executada

- `docker build -t xmqr-client:0.4.0 .`: PASS.
- `docker run --rm xmqr-client:0.4.0 --version`: PASS — `mqtt-client 0.4.0`.
- Imagem final local: 134,839,706 bytes.
- Pub/sub com ambas as imagens na rede isolada Docker: PASS.
- `validation/C30-docker.py`: stdin binário em `--line-mode`, duas mensagens
  confirmadas pelo broker e JSONL sem perda do byte `0xff`: PASS.
- Publicação retained e recuperação após recriar o container do broker com o
  mesmo volume, recebida pela imagem cliente: PASS.

## Limites

Os testes de containers usaram `open-lab` somente no loopback compartilhado para
provar integração e persistência. A imagem não inclui broker; `--open-lab` não
alcança outro container via rede comum. A integração não demonstra mTLS de
produção, terminal interativo PowerShell, multi-arquitetura, assinatura da imagem
ou publicação em registry. `v0.4.0` é apenas etiqueta local da imagem, não tag Git.
