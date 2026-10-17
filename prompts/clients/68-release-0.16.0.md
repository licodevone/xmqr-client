# C68 rev1 — prerelease experimental client 0.16.0

Entrada: `main` C67 `dcc4f4e`, árvore limpa, versão publicada 0.15.0. P127/C66 CIs PASS, P128/C67 CIs pendentes. Broker parceiro P128 em `564e373`.

Objetivo: publicar checkpoint experimental 0.16.0 do client independente com Clean Start e Session Expiry 0/finita/infinita após CI funcional PASS. Elevar Cargo/lock, Dockerfile, changelog, README e nota de release, sem alterar instalações WSL nem o admin independente.

Aceite: gates fmt/test/Clippy/build, integração cruzada, inventário MIT e contratos de distribuição no commit exato; pacote Ubuntu26 amd64 e imagem Docker numerada verificados sem instalar no WSL do operador; tag anotada v0.16.0; prerelease GitHub com assets, SHA-256, manifesto e limite de retomada QoS em novo processo. Publicar Docker Hub numerado somente após digest remoto conferido, nunca mover latest. Fase 6 permanece aberta.
