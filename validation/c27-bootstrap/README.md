# C27 — Crate de validação independente

Este diretório foi criado vazio depois de persistir o prompt C27. Os fontes
args/security/format/engine/main foram redigidos nesta etapa; nenhum src/ do
cliente atual ou broker foi copiado. Configuração Cargo.toml/Cargo.lock, LICENSE
e fixtures C26 foram reutilizados declaradamente, com hashes em inputs.json.
Mesmo autor já conhecia C26; não é reconstrução cega nem equivalência bit a bit.
Não substituir o cliente de produção por este artefato. Versão0.7.0/publish=false
são dados de compatibilidade, não release ou novo produto.

Execute os gates dentro desta pasta: cargo fmt --all -- --check;
cargo test --locked --offline; cargo clippy --locked --offline --all-targets
-- -D warnings; cargo build --locked --offline. Cache/lock são insumos externos
declarados; nenhum download/instalação é necessário no ambiente testado.

A fixture wire teve somente três asserções de vazio adaptadas ao Clippy1.99.
O hash inicial permanece em inputs.json; mudança aprovada em approved_asset_changes.json.
Execute python check_provenance.py para comparar fontes de produção e insumos.
provenance.json guarda hashes, sem pretender provar autoria independente só por hash.

Diferenças deliberadas: módulos/estruturas internas novos, help e texto de erros
de validação não idênticos, quantidade/organização de unitários diferente. Flags,
status de saída, JSON/texto e contratos wire foram comparados nos cenários do registro.
Linux valida segredo0600 em filesystem nativo; não enfraquecer segurança para DrvFS.
Fixtures TLS são peers de teste; não implicam integração secure real com o broker.

No cliente de produção, somente as asserções do teste wire mudaram nesta etapa;
seus src/ permaneceram iguais aos hashes de entrada C27. Não houve instalação,
commit/push/tag/release nem mudança de versão.
