# Changelog

## 0.3.0 — candidato não publicado

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
