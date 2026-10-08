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
