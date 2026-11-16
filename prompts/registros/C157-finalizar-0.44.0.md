# C157 — fechamento do Client 0.44.0

2026-11-08, America/Sao_Paulo. [Prompt C157](../clients/157-finalizar-0.44.0.md).
Base `312ea86`, com C156 incorporado à fonte de validação.

- Adicionados cenários wire de echo com marcadores distintos e reconexão
  com o mesmo ID. `.deb` e imagem verificam `bench --help` na CI.
- PASS local: cinco testes de CLI/configuração com binário direto C156.
  O compilador direto aceitou as fixtures de bench, sem rodar a rede.
- BLOCKED: Cargo offline não encontrou `windows-permissions`; o loopback
  permanece indisponível para integração. Gates CI, pacote, Windows Service
  e publicação aguardam ambiente com acesso ao GitHub.

- Fonte de validação: `46f96b51336b2416c35f47dfb130510b0eef2e54`.
  O push abaixo falhou com `Could not resolve host: github.com`. A CI
  remota não foi disparada por este envio. Tag e release aguardam os gates.

```sh
git push origin 46f96b51336b2416c35f47dfb130510b0eef2e54:refs/heads/release/validation-client-0.44.0
```
