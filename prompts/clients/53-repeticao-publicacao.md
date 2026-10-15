# C53 rev1 — publicação repetida com intervalo (C04)

2026-10-15. Entrada: client0.12.0 publicado; C52 local validado por 41 testes
unitários, 30 wire, Clippy, build e inventário Ubuntu26. `pub` envia uma vez ou
lê linhas de arquivo/stdin; não possui repetição programada.

Objetivo: `--repeat N --repeat-interval-ms M` permite publicar uma única fonte
`--message` ou `--message-file` N vezes (N=1..1000, M=1..60000). M é obrigatório
para N>1; `--repeat` ausente preserva publicação única. Cronometrar no relógio
monotônico sem acumular fila de publicações: iniciar a próxima somente depois
do ACK MQTT pertinente ou do envio QoS0 e do intervalo. Ctrl+C/total timeout
interrompem a espera sem enviar a próxima.

Escopo: parser e runtime do client, fixtures wire, README, CHANGELOG, matriz e
registro. Rejeitar `sub`, stdin e `--line-mode` com repetição programada para
evitar leitura infinita/ambiguidade de lote. Arquivo é lido uma vez antes da
conexão; bytes e limites existentes se preservam. Não reenviar após erro de
ACK, queda de conexão ou resultado desconhecido. Nenhuma alteração no broker.

Compatibilidade: wire MQTT3.1.1, perfil TOML e uso sem flags preservados.
Saída `publish_complete` é emitida uma vez por tentativa confirmada; status não
indica sucesso de negócio. Testar QoS0/1/2, contagem, intervalo, validações e
cancelamento, além de fmt/test/Clippy/build/inventário. Não marcar a fase 5
inteira por C53.
