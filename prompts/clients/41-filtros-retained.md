# C41 rev1 — filtros retained na saída do assinante

Data: 2026-10-12. Entrada: HEAD 8bb7aaf, client 0.7.1; broker 0.11.2.
Arquivos Docker não rastreados preservados. Pedido autoriza evolução, tags e
releases após aceite; o harness de carga continua separado da CLI.

Objetivo: `sub --ignore-retained` omite publicações com RETAIN=true;
`sub --retained-only` exibe somente essas publicações. Default exibe todas.
Flags sem valor, mutuamente exclusivas, recusadas em pub e se repetidas.

Escopo: src/cli.rs, src/session.rs, tests/wire.rs, validation/C41-integration.py, README, CHANGELOG,
docs/mosquitto-parity.md, registro C41 e manifests do marco 0.8.0 após aceite.

Contrato: filtro local de saída, não altera SUBSCRIBE, ACL, recepção, ACKs ou
reconexão. `--count` conta mensagens exibidas; mensagens filtradas também
completam QoS1/2. RETAIN é a flag recebida: publicação ao vivo normalmente
chega RETAIN=false mesmo quando o publicador usou retain. Nenhuma dedução da
existência de retained armazenado. text/jsonl usam a mesma política.
Preservar TLS/SAN, credenciais privadas, quotas, cancelamento e deadlines.

Aceite: CLI negativos, fixtures wire com mistura retained/live, count filtrado,
QoS1/2 filtrado até conclusão e cancelamento; integração com broker separado
para retained bootstrap e publicação ao vivo. Gates fmt/test/clippy/build em
Ubuntu26. Revisar prompt antes de corrigir falhas e registrar resultados reais.

Compatibilidade/migração: flags opt-in, default preservado, sem alteração wire
ou estado. Marco funcional client 0.8.0 com tag anotada no commit validado;
release somente com artefatos verificados. Broker não exige nova versão por
este filtro. Atualizar checklist central sem declarar fase 5 concluída.

## rev2 — ajuste após gate Clippy

Windows: 30 unitários e 21 wire PASS, build release PASS. Clippy recusou
conversão usize→u8 na fixture; substituir por conversão verificada sem relaxar
lints e repetir Clippy/testes. Tentativas WSL com bash de login falharam na
conexão; /bin/true voltou a responder. Validar usando shell sem perfil antes
de marcar gates Ubuntu como concluídos. Nenhuma tag criada com gate falho.
