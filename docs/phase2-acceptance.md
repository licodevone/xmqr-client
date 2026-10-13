# Client — checklist da fase 2

2026-10-13, C43/C44 e P85 físico. Runtime0.8.1 preservado; incremento de testes/documentação,
não novas opções CLI. O client participa do checklist, não apenas o broker.

- [x] ~~54testesRust, fmt, Clippy, buildrelease e inventário de licenças.~~
- [x] ~~5contratosPython: bytes/QoS/retain/duplicatas/CLI/relatório existente.~~
- [x] ~~Mosquitto2.0.22 pinado: pub/submTLSQoS0/1/2, binárioNUL/0xff/CRLF.~~
- [x] ~~Retained replay/delete seguido de entrega ao vivo.~~
- [x] ~~Will porSIGKILL do processo próprio e retomada offline persistente.~~
- [x] ~~CA/senha/SAN inválidos e ACLXMQR negada recusados.~~
- [x] ~~ESP32 físico: P83/P84/P85 na COM4, Will/reconexão/negativos/retained restart/MosquittoQoS2 e memória-probe; Docker só hospeda servidor/cliente externo.~~

[59cenários reais](results/c43-mosquitto-mtls-2026-10-13.json).
Fontes da referência e dos runtimes fixadas porimageID/labels; o HEADclient
no relatório é a base de entrada do harness anterior ao commit, não um
rebuild da imagemv0.8.1. Correção broker0.11.4 terá campanha final nos anexos.

[Aceite global](https://github.com/licodevone/xmqr/blob/main/docs/phase2-acceptance.md)
está ACCEPTED apósP85/E01. MOSQUITTO3.1.1 não é conformidade integral/MQTT5.
MIT/dependências, TLS/SAN, quotas, reconexão e saída preservados. Não criar
nova versão de client apenas por este harness. Comando reproduzível:
python scripts/verify_mosquitto_phase2.py --hash-file PUBLIC_FIXTURE_HASH
--openssl OPENSSL --report target/c43-novo.json (argumentos numa linha).
Somente hashfixture público; não usar arquivo da instalação real.

P85:14provas externas,0violações/cleanup, Wi-Fi9048ms/broker7869ms e probe
110592bytes estável. Memória/probe não é heap total/min livre ou vazamento longo;
negativosSDK usam callback genérico contrastado com baseline válida e serviço
vivo. Sem alegar pcapTLS/energia/produção. [Evidência física](https://github.com/licodevone/xmqr/blob/main/docs/results/p85-e01-campanha-2026-10-13.json)
e [registroC44](../prompts/registros/C44-fase2.md). Runtime comparado à tagv0.8.1:
idêntico; .deb/imagem existentes verificados, sem retag/rebuild desnecessário.
