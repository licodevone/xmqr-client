# Registro C43 — interop real da fase 2

Data2026-10-13, prompt43rev1.5. Basebc94bd0/client0.8.1; brokerbase0.11.3.
Pedido vigente autoriza implementação, commits/push e releases de marcos aceitos.
Duas archivesclient0.7.1 não rastreadas preservadas, runtime/dep/MIT inalterados.

PASS:54testesRust, fmt/clippy/buildrelease/inventário,5contratosPython;
47cenários externos básicos e59na ampliaçãofinal. DockerLinux/amd64 pinado:
Mosquitto2.0.22sha256:199ea8ef2e35ec2b1b37e59cfd1dbae538ed4dfa4a2251a121a52215a6248a21;
XMQRruntime e labels íntegros no JSON. CA/chaves/senhafixtures privadas
temporárias, namespacesloopback sem porta host publicada.

Matriz:2brokers×2direções×3QoS×live/replay/delete=36;8negativosCA/senha;
3ACLXMQR;4Will;4sessõespersistentes/offline;4SANservidor errado=59.
Payloadbinário conferidoNUL/0xff/CRLF, QoS/retained/count e ACKterminal.
WillSIGKILL só processo próprio; não prova energia. Retomada não prova
persistência local de client, power loss ou reconexão sob todas as falhas.

Falhasrevisadas: readinessMosqexit7 porClientID omitido (contratoXMQR IDnão vazio),
fix -i explícito; fixturepersistente24bytes ultrapassavaCLI23, corrigidoID16bytes.
Nenhum contrato de produção foi ampliado para acomodar fixture errada.
Todos os containers/volumes criados são removidos no finally, sempruneglobal.

Resultado de software aceito; E01ESP32físico BLOCKED/NOT_RUN. Fase2global
continuaPARTIAL/BLOCKED. Não criar nova versão/tagclient por scripts/docs.
Registro anterior ao commit; HEAD no JSON é a base do harness em execução,
image revision identifica o runtime realmente testado. Repetição com broker
corrigido0.11.4 fica nos anexos da release, sem mover clientv0.8.1.
