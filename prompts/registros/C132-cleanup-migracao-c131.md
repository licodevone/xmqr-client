# C132 — integrar teardown aceito sem absorver C128

2026-10-18. PromptC132 commit04f3334 precedeu alterações. Somente os dois
scripts de teste foram integrados da fonte C1312153ba9; gitdiff--no-index
entre cada arquivo e a cópiaC131 final não apresentou diferença.

Teste real Ubuntu26/WSL, Python3.14, TMPDIR=/var/tmp:
`scripts/test_c131_migration_cleanup.py --legacy-client <ELFv1> --client <ELF038>`.
PASS0,2733s: metadados originais preservados, leitura/replay não regravaramv1,
conclusão gravou versão2, sem saída duplicada, cleanupPASS e nenhuma criança
viva no teardown da pasta. Markers C129_V1_MIGRATION_PASS e
C131_MIGRATION_CLEANUP_PASS observados. SIGKILL próprio, não perda de energia.

LegacyELF SHA256
`109ec97326500b8edfe9c54f09318103f661866a3e16927a9b63e8d4f473d3e9`;
client038ELF SHA256
`a5ccc20519fbe22661e230956d657379452bec69e1829dd18047d30556f85703`;
harness SHA256
`a2aa613408ec603a2888b2408c39f7e0a9f6b0b908f130e629d3c8ccf0c033c3`.

Sem alteração de Rust/Cargo/versão/CI/Docker/README/bridge originais. C128dirty
continua separado; não houve merge do branchrelease038, retag ou nova versão
por correção só de teste. A release038 é independente em branch com fonte
fca6fc8. Este resultado não encerra aceite conjunto broker/admin/client6.
