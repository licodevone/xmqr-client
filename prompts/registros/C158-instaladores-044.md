# C158 — empacotamento do Client 0.44.0

2026-11-08, America/Sao_Paulo. [Prompt C158](../clients/158-instaladores-044.md).

- Fonte isolada `46f96b51336b2416c35f47dfb130510b0eef2e54`.
- BLOCKED MSI: `cargo metadata --locked --offline` para o manifesto de
  serviço não encontrou `windows-permissions` no índice local. O fetch
  online do mesmo manifesto falhou ao resolver `index.crates.io`.
- BLOCKED DEB: WSL e Docker Desktop recusaram acesso ao ambiente Linux.
- Nenhum instalador Client gerado. Nenhuma instalação, tag ou release.

## Reavaliação em 2026-11-14

- PASS: o executável de diagnóstico C156 retorna 0.44.0, e os cinco
  testes de CLI/configuração passaram novamente.
- FAIL de execução: os três cenários bench da fixture C157 terminaram
  com connect/transport_io. Um controle Python independente com listener
  local confirmou WinError 10013 ao conectar ao loopback. A integração
  continua bloqueada pelo ambiente, sem novo aceite de rede.
- BLOCKED: novo fetch locked do manifesto de serviço falhou ao resolver
  index.crates.io para obter windows-permissions. WSL e Docker também
  recusaram acesso. MSI, DEB, CI e release final continuam pendentes.
- O template MSI compartilhado no Broker recebe a correção P283 da ordem
  de remoção da versão anterior. A validação do Client exige seu pacote real.
