# Build experimental do serviço

Manifest independente, sem workspace, compartilhando `../../src/main.rs`.
Dependências normais e versão devem acompanhar o Cargo.toml da raiz; somente
este manifest acrescenta windows-service e habilita windows-service-runtime.

É necessário obter dependências, gerar/revisar Cargo.lock nesta pasta e então
executar fmt/test/clippy/build --locked neste manifest. O binário válido responde
a `--windows-service-capabilities` com JSON schema=1, component=client e
service_entrypoint=true. O binário da raiz não possui o adaptador.

Estado atual e limitações: [guia](../../docs/windows-service.md).
Nenhum Cargo.lock experimental foi fabricado durante indisponibilidade da rede.
