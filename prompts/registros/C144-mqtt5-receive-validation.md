# C144 — execução das quatro recomendações de recepção MQTT 5

Data do pedido: 2026-10-18, America/Sao_Paulo.
Autoridade: «Resolva essas recomendações»; [prompt v1.1](../clients/144-mqtt5-receive-validation.md).
Base: `1e73717d37649b2e423b72fd823245991f3fb7f2`, client 0.42.0/bridge 0.4.0.
Validação da árvore local modificada, sem novo commit/tag/release. A versão
impressa não identifica uma distribuição nova. O diretório não rastreado
`vendor/rumqttc-0.25.1/target/` já existia e foi preservado.

## Alteração e interpretação

O leitor MQTT 5 comum valida UTF-8 e rejeita U+0000 em strings, incluindo
User Property e o tópico PUBLISH armazenado em bytes. O leitor VBI comum
exige largura mínima, incluindo comprimentos de propriedades e Subscription
Identifier. O framing valida flags reservadas antes de aguardar o corpo;
PUBREL/SUBSCRIBE/UNSUBSCRIBE preservam 0x2. Sem parser novo, realocação por
comprimento não validado ou alteração em payload/Correlation Data binários.
U+FEFF, caracteres Unicode e strings vazias permitidas continuam intactos.

São as quatro recomendações Client SHOULD de §4.13.1 identificadas por P232;
o código segue as representações válidas de §§1.5.4/1.5.5/2.1.3 da
[norma OASIS MQTT 5.0](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html).
Não se reclassificou recomendação como MUST do Server. O decoder MQTT 3.1.1,
TLS/SAN/CA, limites existentes, contratos duráveis e licenças foram preservados.
Regressões exercitam API pública e bytes independentes do encoder do produto.

## Reprodução e resultados

Ubuntu 26.04/WSL, Rust/Cargo 1.99.0, Python3, TCP loopback, portas efêmeras,
processos com prazo e cleanup. Build cache nativo privado em
`/var/tmp/xmqr-client-p92-native-e688c75`; recursos temporários independentes.
Os comandos Rust rodaram com CARGO_TARGET_DIR apontando para esse cache.
Executável usado explicitamente: `.../debug/mqtt-client`.

| Gate/campanha | Resultado real |
| --- | --- |
| P232 original antes do patch, debug 0.42.0 | FAIL: 8 PASS/4 FAIL/0 BLOCKED, recomendações reproduzidas |
| Novos vetores Rust antes do patch | RED esperado: 2 PASS/5 FAIL, compilação válida |
| Novos vetores Rust após o patch | PASS: 7/7, incluindo VBI em quatro larguras, flags, Unicode/binário e fragmentação/concatenação |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `cargo test --locked` | PASS: 200 testes (7 bridge + 112 client + 7 codec novos + 5 QoS2 + 1 WS + 68 wire) |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS, exit 0 |
| `cargo build --locked --bins` | PASS, exit 0 |
| `python3 scripts/license_inventory.py --check` | PASS: declaração MIT/inventário preservados |
| P232 original, sem alterar seu oráculo | PASS: 12/12, 0 FAIL/0 BLOCKED |
| Nova fixture C144 do executável | PASS: 14/14, rejeição rápida sem entrega/ACK de sucesso e controle Unicode/binário |
| C140–C142 obrigações/controles existentes | PASS: 42/42 |
| C129 journal inbound MQTT 5 | PASS: todos os 21 cenários, inclusive crash/recuperação e ACK após commit |
| C129 compatibility | PASS: seis módulos TCP/WSS mTLS, outbox QoS1/QoS2 e assinatura durável |

Não houve falha inesperada dos gates após o patch. Os FAILs de reprodução
foram preservados e precederam a revisão v1.1 e a correção. A nova fixture
usa helper C140; ambos têm hashes no relatório. Assegura ausência de saída
de sucesso e ACKs para as entradas inválidas e fechamento antes de timeout.
O P232 original foi executado da pasta irmã do broker sem modificá-lo.

Relatórios completos, hashes de binário/fontes/fixtures e resultados estão em
[evidência C144](../../docs/conformance/mqtt5-receive-validation-c144.json).
O campo `source_commit_declared` dos relatórios P232 identifica a base; o
candidato possui alterações locais e é identificado pelos hashes adicionais,
não alegado como um binário reconstruído de commit limpo ou release 0.42.0.

## Aceite e limites

PASS local das quatro recomendações e das regressões indicadas. A CI recebeu
a fixture para debug e ELF extraído do pacote, com upload do relatório mesmo
em falha; essa configuração não é alegada como uma execução remota.
Não houve nova CI, pacote, imagem, instalação, publicação ou teste do serviço
real. A publicação 0.42.0 e os ledgers históricos P232 continuam imutáveis.
Não houve homologação nativa Windows, fuzzing, todos os servidores externos
ou declaração de conformidade integral. Nenhum gate local listado ficou
BLOCKED. Estado, credenciais reais e os repositórios broker/admin não mudaram.
