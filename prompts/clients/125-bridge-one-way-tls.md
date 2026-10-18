# C125 — sidecar bridge unidirecional TLS

Entrada: client 0.36.0 publicado, broker 0.42.0 publicado, P223 define arquitetura da fase 8. Este prompt precede código funcional.

Objetivo: adicionar binário experimental `mqtt-bridge` separado da CLI `mqtt-client`, com configuração TOML v1 privada para duas pontas TLS/mTLS e identidades distintas. Assinar um filtro QoS 0 na origem e publicar QoS 0 no destino com substituição estrita de prefixo. A operação deve continuar por reconexões limitadas sem acumular fila indefinida.

Validação: caminhos de CA/certificado/chave e senha em arquivo privado; não aceitar TLS sem CA ou identidade quando mTLS for configurado. Recusar mesma ponta, IDs iguais, wildcard inválido, prefixo alvo inválido, mapeamento que não preserve tópicos válidos, tamanhos acima do limite e mais de uma direção. Não imprimir payload, senha ou caminhos privados em falhas. CLI `--version`/`--help` antes de ler configuração.

Aceite: unitários de configuração/mapeamento e integração real XMQR↔Mosquitto em duas topologias, publicação em tópico remapeado, queda e reconexão com limites, negativos CA/SAN/credencial/ACL, TLS e TCP não regredidos, fmt/test/Clippy/licença/CI. Apenas QoS 0 sem retained nem persistência; fase 8 continua aberta.
