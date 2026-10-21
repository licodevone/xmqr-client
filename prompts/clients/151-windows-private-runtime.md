# C151 - Runtime nativo privado Windows (v1.0)

Data: 2026-10-21. Continuidade C150; sem instalar servicos no host.

Implementar senha em arquivo privado Windows usando somente Rust seguro no codigo
proprio e APIs nativas encapsuladas por dependencia fixa. Validar dono/DACL no
handle aberto, recusar permissoes amplas e componentes reparse; preservar Linux.
Habilitar pub/sub em servico com entrada nao interativa, saida NDJSON privada,
limite de armazenamento e ACK manual somente depois de flush/sync_data da saida.
Sincronizar as correcoes SCM do broker sem acoplamento de runtime entre projetos.
Fornecer --service-config para executar a mesma preparacao em console e testar
TLS/MQTT sem registrar o executavel no SCM. Configuracao, senha e saida devem
usar caminhos absolutos e arquivos/parents privados. Sem senha em argv/config/log.

Gates: build MSVC nativo, testes reais de DACL/reparse/senha/limite de saida,
Clippy estrito, formato e smoke de capabilities. Integracao TLS/MQTT e empacotamento
coordenados no broker. Nao declarar aceite SCM sem execucao em VM Windows.
Sem commit/push, instalacao, mudanca global de permissao ou enfraquecer defaults.
