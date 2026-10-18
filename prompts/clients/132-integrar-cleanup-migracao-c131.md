# C132 v1.0 — integrar correção do harness de migração

Preparado em2026-10-18 antes de alterar arquivos. Originalmainf3c7162 contém
C129 aceito, mas trabalhoC128dirty de distribuição/bridge permanece reservado
ao seu escritor. C131 publicou prerelease038 em branch separado e corrigiu
o teardown do harness após RED determinístico/CI. Não fazer merge do branch
inteiro nem absorver Cargo/CI/Docker/README/bridge modificados no original.

Integrar somente scripts/verify_c129_v1_migration.py e o novo
scripts/test_c131_migration_cleanup.py a partir de2153ba9, preservandoMIT,
oráculos e todos os bytes fora desses dois arquivos. Adicionar registroC132.
Sem mudança de runtime/manifest/versão, sem tag artificial ou retag038.

Falha corrigida: remover TemporaryDirectory antes de encerrar filho ainda
vivo pode causar OSError durante teardown apesar de metadados MQTT corretos.
ExitStack deve limpar filho antes listener/pasta, inclusive em exceção.
Reexecutar contrato real com ELFv1 congelado e ELF038 publicado, explicitando
hashes/tempos. Checar igualdade dos arquivos com fonteC131 validada e manter
diffC128dirty intacto. Commit/push só dos arquivos próprios; nenhum serviço,
estado ou credencial do operador será tocado.
