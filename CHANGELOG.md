# Changelog

## 0.4.0 — candidato local, tag sugerida v0.4.0

- C29: `sub` aceita até 256 filtros por repetição de `--topic`, numa assinatura.
- Pacote SUBSCRIBE limitado a 64 KiB; SUBACK exige uma concessão por filtro.
- `pub` continua aceitando um único tópico; mesmo QoS se aplica ao lote.
- Integração C29 exercita filtros disjuntos/sobrepostos, retained e `$` com XMQR.
- Evidência: `prompts/registros/C29-multiplos-topicos.md`.
- Gates Windows e Ubuntu-26.04/WSL aprovados; Rust 1.88 exato não validado.

## 0.3.0 — tag publicada

- C28: --message-file e --stdin preservam payload binário até4096 bytes.
- --line-mode publica serialmente por LF/CRLF, preserva linhas vazias e EOF sem LF.
- Backpressure, espera de entrada limitada8s e cancelamento sem travar runtime.
- Conclusões text/JSONL por linha; falha parcial não reenvia mensagens.
- --version e manifest/lock alinhados à versão própria0.3.0; dependências sem upgrade.
- Próxima tag sugerida: **v0.3.0**. Não criada/enviada por esta tarefa.
- Evidência: prompts/registros/C28-payload-binario.md; MSRV1.88 e interoperabilidade
  externa completa não comprovados.

## v0.2.0 — tag existente, commit6d25b33

- Inclui C26 (reconexão opcional limitada e JSONL) e C27 (contrato/ensaio bootstrap).
- O manifest nesse tag ainda indicava0.7.0 herdado da origem. C28 corrige adiante,
  sem reescrever tags/histórico. Existência da tag remota não comprova GitHub Release.

## Extração inicial

- Cliente Rust2024/MIT independente do broker; binário mqtt-client preservado.
- Versão0.7.0 herdada, descrita historicamente em ORIGIN.json.
