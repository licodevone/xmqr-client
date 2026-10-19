# C146 — distribuição corretiva client 0.42.1

Prompt [C146 v1.0](../clients/146-release-0.42.1-receive-validation.md)
publicado antes das alterações em `e980d28`. Runtime C144 `25c2e8a`
com [CI anterior PASS](https://github.com/licodevone/xmqr-client/actions/runs/39194165930).
C145 retirou arquivos gerados do índice preservando os 1.861 arquivos locais.

Candidata: client 0.42.1/bridge 0.4.0; sem mudança funcional além de C144.
CI, pacote, imagem, lifecycle, tag/release e hashes remotos desta candidata
ainda pendentes. Nenhum desses gates é declarado PASS por preparação.
Broker 0.43.0/admin 0.11.0 e releases anteriores permanecem independentes.

## Aceite final e publicação em 19/10/2026

O parágrafo anterior registra a preparação; os resultados finais são:

- Fonte exata `db844a0808dfa82e436e5b0fe960d66536df4703`,
  [CI `39257417702`](https://github.com/licodevone/xmqr-client/actions/runs/39257417702)
  com os cinco jobs PASS: Rust, pacote Ubuntu, imagem, integração da bridge
  debug e integração dos binários extraídos do pacote.
- C140–C142: 42/42 no debug CI, pacote CI e ELF da imagem local; C144:
  14/14 nos mesmos três contextos. Relatórios baixados da CI foram lidos,
  não se inferiram os resultados só do nome dos jobs. P232 original inalterado
  passou 12/12 no ELF do pacote CI; nenhuma falha histórica foi apagada.
- `.deb` obtido da CI, Package xmqr-client, Version 0.42.1-1, Architecture amd64,
  SHA-256 `c6a1356e5eaefda6be2f18abb5dd5fafe4865c166346a6fc57002cd99cedfb7c`.
  Binários imprimem `mqtt-client 0.42.1` e `mqtt-bridge 0.4.0 (xmqr-client 0.42.1)`.
- Imagem local compilada da mesma fonte, Linux/amd64, usuário 10001:10001,
  revisão OCI exata; duas versões verificadas. Build terminou PASS.
  O tar e o manifesto foram exportados de HEAD limpo, sem builds vendorizados.
- Tag anotada `v0.42.1`, objeto `1b3dbcafaac03145db561a6eb44526b63e5995df`,
  alvo db844a0 completo acima, ambos conferidos por ls-remote.
  [Prerelease](https://github.com/licodevone/xmqr-client/releases/tag/v0.42.1)
  com seis anexos: todos baixados e idênticos aos locais por SHA-256;
  os três sidecars baixados passaram sha256sum -c.
- Imagem numerada enviada ao Docker Hub sem mover latest; digest remoto
  `sha256:aa996059df5c0acb61e383c3b30df7c0816f3ce0a12d7a7b5c14f229d22a5b53`.
  Pull por digest e execução dos dois binários read-only/network-none PASS.

## Falha local preservada e limites

A tentativa extra do lifecycle local retornou TimeoutExpired após 180s,
sem prova de concluir instalação/rollback. Não se declara PASS local. O
contêiner descartável `ac24f2bc1439` já não existia na verificação posterior.
Os mesmos gates de versões, configuração preservada, unit opt-in e ciclo
0.42.0→0.42.1→0.42.0→0.42.1 passaram na CI da fonte exata; esse é o aceite
do lifecycle. O pacote antigo foi conferido contra o SHA publicado
`552db5568fde4c045df5c1e8bfe5d5c98a1a5073ac201ada218d977d1e8f54b8`.
Não se alterou produto/oráculo para encobrir o timeout, nem se repetiu esse
ensaio adicional sem necessidade. Não há prova da causa precisa da demora.

Docker Desktop foi iniciado para o ensaio e retomou contêineres preexistentes
conforme suas políticas automáticas; não se alteraram suas configurações nem
se pararam esses contêineres. Nenhum pacote foi instalado no host do operador.
Três CIs intermediárias só de prompt/higiene foram canceladas como superadas;
a CI integral da candidata acima foi preservada e terminou PASS.

[Evidência de distribuição C146](../../docs/conformance/client-release-0.42.1-c146.json)
reúne manifesto, hashes remotos, resultados e origem dos relatórios. Fases 8/9,
homologações ampliadas e lacunas restantes da auditoria continuam abertas.
