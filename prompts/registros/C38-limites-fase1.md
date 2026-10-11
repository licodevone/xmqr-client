# C38 — limites da fase1

2026-10-11. Entrada HEAD6ee5aa87d0e1eb16c48c625408eeaf227197da14, versão0.6.0, árvore limpa; prompt38-limites-fase1 rev1 apresentado antes do código, rev2/3/4/5 antes das correções correspondentes. Sem alteração de manifesto/lock, commit/tag/publicação pelo agente desta etapa. Skill local xmqr-client-development aplicada.

## Implementado

- Mensagem de SUBSCRIBE esclarece pacote completo64KiB. Unitário verifica65536 aceito/65537 recusado e UTF-8 por bytes.
- Fixture4096bytes preserva binário emQoS0/1/2;4097 já é recusado antes de rede nos negativos existentes.
- SUBACK0x80 explica ACL/limites como itens a conferir, sem atribuir causa não informada porMQTT3.1.1. Recusa de quota do broker atual fecha transporte; harness não exige SUBACK nem interpretaEOF como reason de quota.
- Incoming rumqttc0.25.1 verifica Remaining Length, não tamanho total. Cap65531 de corpo reserva header máximo5, inclusive encoding não mínimo. Pode recusar frame canônico65536 por1byte; saídaSUBSCRIBE permanece65536 inclusivo. Essa exceção é explícita no README/matriz/changelog e não afeta packets broker suportados payload4096/tópico1024. Sem vendor/fork ou mudança de TLS/retry.
- Fixtures de header mínimo/não mínimo verificam65531 aceito para aguardar corpo,65532/65533 recusados antes do corpo; stdout vazio, sem exposição de payload.
- scripts/verify_phase1.py exige --broker/--client explícitos, Unix, loopback e estado/processos temporários. Stdout dos assinantes vai para arquivos temporários próprios, sem pipe que possa bloquearJSON4096; stderr tem um único leitor de readiness. Cobre real4096/4097 emQoS0/1/2, keepalive1 funcionando por3s de inatividade com timeout1, SUBSCRIBE65536/65537 e recusa de assinaturas. Não foi executado contra broker nesta plataforma.

## PASS — Windows portável

Rust/cargo1.98.1 (MSRV1.88 exato não instalado). Cargo fmt --all -- --check, cargo test --locked (29unitários+20wire,0falhas/0ignorados), cargo clippy --locked --all-targets -- -D warnings, cargo build --locked --bins e git diff --check aprovados no diff final. Python3.12.14 AST do harness aprovado. Logs locais ignorados em target/c38-validation/{fmt,clippy,test,build}-final.log. Mensagens de linker informativas não impediram gates.

## FAIL inicial, resolvido

Clippy encontrou3casts potencialmente truncados/perda de sinal em fixtures. Registro/rev3 do prompt preparados antes da correção para try_from com valores limitados. Clippy final PASS. Revisão cruzada detectou encoding não mínimo do Remaining Length e expectativa incorretaSUBACK em quota; rev3/4 ajustaram teste/diagnóstico/harness antes da repetição de gates. Revisão posterior identificou risco de deadlockJSON~15KiB em stdoutPIPEUnix antes de wait; rev5 preparada antes da correção para arquivo temporário. PythonAST e guardWindows repetidos PASS/BLOCKED respectivamente, sem alteração Rust ou novo aceiteUnix.

## BLOCKED — aceite Unix/fase1

Harness chamado noWindows retorna2 com mensagem BLOCKED antes de executar broker. Distribuição Ubuntu26 do usuário não acessível à identidade desta sessão; C38 contra broker real, quotas/recovery P55, pacoteUbuntu e instalação/upgrade/rollback permanecem sem execução atual. Nenhum serviço/estado instalado foi tocado. Fixtures Windows não comprovam esses gates. Fase1 permanece aberta até evidência real.

Comando em checkoutUnix isolado após build dos dois projetos:

```bash
python3 scripts/verify_phase1.py --broker ../xmqr/target/debug/mqtt-broker --client target/debug/mqtt-client --report /tmp/c38-phase1.json
```

Teto64KiB para entrada não significa fronteira inclusiva idêntica à saída: cap conservador e exceção1byte são parte do contrato deste incremento, não conformidade completa com peers externos/Mosquitto. Release/tag de fase1 só após gates faltantes; decisões de publicação pelo coordenador.
