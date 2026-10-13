# Client — checklist da fase 2

2026-10-13, C43. Runtime0.8.1 preservado; incremento de testes/documentação,
não novas opções CLI. O client participa do checklist, não apenas o broker.

- [x] ~~54testesRust, fmt, Clippy, buildrelease e inventário de licenças.~~
- [x] ~~5contratosPython: bytes/QoS/retain/duplicatas/CLI/relatório existente.~~
- [x] ~~Mosquitto2.0.22 pinado: pub/submTLSQoS0/1/2, binárioNUL/0xff/CRLF.~~
- [x] ~~Retained replay/delete seguido de entrega ao vivo.~~
- [x] ~~Will porSIGKILL do processo próprio e retomada offline persistente.~~
- [x] ~~CA/senha/SAN inválidos e ACLXMQR negada recusados.~~
- [ ] ESP32 físico: validar consumidor externo, não substituir por Docker.

[59cenários reais](results/c43-mosquitto-mtls-2026-10-13.json).
Fontes da referência e dos runtimes fixadas porimageID/labels; o HEADclient
no relatório é a base de entrada do harness anterior ao commit, não um
rebuild da imagemv0.8.1. Correção broker0.11.4 terá campanha final nos anexos.

[Aceite global](https://github.com/licodevone/xmqr/blob/main/docs/phase2-acceptance.md)
continua PARTIAL/BLOCKED atéE01. MOSQUITTO3.1.1 não é conformidade integral/MQTT5.
MIT/dependências, TLS/SAN, quotas, reconexão e saída preservados. Não criar
nova versão de client apenas por este harness. Comando reproduzível:
python scripts/verify_mosquitto_phase2.py --hash-file PUBLIC_FIXTURE_HASH
--openssl OPENSSL --report target/c43-novo.json (argumentos numa linha).
Somente hashfixture público; não usar arquivo da instalação real.
