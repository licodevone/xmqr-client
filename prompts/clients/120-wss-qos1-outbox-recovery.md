# C120 — retomada da outbox QoS 1 após PUBACK perdido em WSS

Entrada: C119 aceito parcialmente em `main`; outbox MQTT 5 QoS 1 por WSS mTLS conserva o journal quando a conexão cai antes do PUBACK, mas ainda não há ensaio de retransmissão pela mesma URL.

Objetivo: demonstrar retomada segura após queda em WSS. Na segunda conexão com mesmo Client ID, URL, operação e sessão persistente confirmada pelo broker, reenviar o mesmo Packet ID com DUP=1, receber PUBACK e só então remover o journal. Se Session Present=0, preservar o journal sem reenvio. URL diferente, WS aberto, QoS 2 por URL e manifesto de assinatura por URL permanecem recusados.

Aceite: fixture mTLS real com dois handshakes e CONNACK controlado, observação dos bytes de Packet ID/DUP, retorno de sucesso apenas após PUBACK, journal inalterado com Session Present=0, regressões C119/TCP/outbox QoS 2/assinatura, suíte/fmt/Clippy/licenças e CI da fonte exata. Registrar limite: uma fixture controlada não comprova sobrevivência do estado do broker em reinício nem todos os interleavings de rede. Sem tag/release isolada.
