# C56 rev1 — prerelease client0.13.0 da fatia C03/C04/C06/C08

2026-10-15. Entrada `main` limpo antes de C52 em
`55e8e462840fc487676d41852f9d0d26edb46d9f`; tag/release v0.12.0
publicada. C52–C55 locais passaram 46 testes unitários/33 wire, fmt, Clippy,
build release e inventário no Ubuntu26. A fase 5 global ainda depende do canal
e das funções administrativas do broker/admin.

Objetivo: publicar client0.13.0 como prerelease independente com exclusão local
de tópicos, publicação repetida, formatos hex/raw e Will binário. Alinhar
Cargo.toml/lock e Dockerfile; atualizar README, CHANGELOG, matriz e registros.
Repetir gates após bump, contratos de distribuição e integração proporcional.
Gerar .deb Ubuntu26 amd64, Docker numerado, exports, checksums e manifesto do
commit limpo. Tag imutável, push, GitHub prerelease e digest remoto somente
após validação. Não mover latest nem anunciar fase 5 concluída.

Compatibilidade: MQTT3.1.1/wire, perfis existentes e defaults preservados.
Documentar upgrade/rollback para v0.12.0 e limites experimentais. Registrar
comandos/resultados reais; falha impede publicar até correção revisada.

## rev2 — integração com broker real

Os testes wire cobrem frames/ACKs, mas a distribuição da fatia C03/C04/C06
deve incluir um broker0.14.0 isolado em open-lab e estado descartável. Adicionar
harness finito com assinante/exclusão, publicação repetida e saída raw/hex;
verificar ausência de replay inesperado e limpar processos/estado ao final.
Sem ler instância instalada ou segredo do operador.
