# Registro C154 — diagnóstico MQTT online

Data local: 2026-11-08, America/Sao_Paulo.
[Prompt C154 v1.1](../clients/154-probe-online.md).
Base `04189b6124aca075dd944b937351eb87380cc47c`.

- `probe` verifica CONNECT/CONNACK em MQTT 3.1.1 ou 5. Assinatura exata
  e ida e volta QoS 0 são opt-in. O relatório usa códigos fixos redigidos.
  A sessão limpa e efêmera evita usar as identidades de sessão dos perfis.
- PASS: compilação direta do binário Windows via rustc 1.99.1 e Clippy
  direto com `-D warnings`, usando dependências compiladas do C151.
  Quatro testes diretos de CLI/configuração passaram. Formatação e diff
  foram verificados. A validação direta é complementar ao Cargo.
- FAIL inicial: colisão de `Outgoing` na adaptação de eventos, corrigida
  após revisão C154 v1.1. O build direto corrigido passou.
- BLOCKED: build/test/Clippy via Cargo exigem `windows-permissions` ausente
  no cache e acesso ao crates.io. A suíte unitária direta também requer um
  artefato Tokio compilado com `test-util`.
- BLOCKED: as três fixtures wire iniciaram, mas não receberam conexão.
  O teste de controle com socket local confirmou WinError 10013. Nenhum
  cenário wire foi aceito. WSL retornou `E_ACCESSDENIED`.

Executar a validação normal do Client e `cargo test --locked --test wire probe`
no Ubuntu. Antes de publicar, repetir os cenários pertinentes no pacote.
