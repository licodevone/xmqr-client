# C156 — benchmark MQTT

2026-11-08, America/Sao_Paulo. Base `312ea86`. [Prompt v1.1](../clients/156-bench.md).

- PASS: compilação direta com dependências C151, Clippy direto com os
  lints do manifesto e cinco testes de CLI/configuração. O teste novo
  cobre limites, argumentos com valor ausente e prefixos indevidos.
- FAIL inicial: Clippy detectou três problemas de estilo e representação;
  corrigidos após revisão do prompt e confirmados no novo gate direto.
- BLOCKED: Cargo offline não encontrou `windows-permissions`. A fixture
  wire compilada expirou sem aceitar conexão, com controle de loopback
  retornando `EACCES`. Capacidade de 1.000 clientes, mTLS e p95/p99 reais
  ficam por medir. Nenhuma execução foi feita contra Broker de produção.
- O candidato congelado 0.44.0 segue em `312ea86`. O novo código fica em
  desenvolvimento, sem nova tag ou release. Push do candidato falhou por DNS.
