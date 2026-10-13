# RegistroC45 — robustez/reconexão / fase3 C07

2026-10-13. EntradaGitd9d97d51b88832044c8d2de3786ce4262ee77ec1/runtime0.8.1,
preservados2arquivostar/SHA0.7.1untracked. PromptC45v1 anterior ao código,
revisado1.1 a1.6 antes de correções/reforços. Skillxmqr-client-development usada
para preservar contratoCLI/TLS/rumqttc e definirgates.

## Resultados durante execução

- FAIL inicialcargo test --locked: E0618 em harnessnovowire (bindinglistener
  ocultavafunção). Corrigido referênciaqualificada apósrev1.1, sem trocaraceites.
- Primeira suíte completaPASS59 (33unit+26wire),12.92s wire/0.13sunit.
- ClippyFAILclippy::assert_is_empty em dois novosasserts; promptrev1.2antes
  assert_eq!/assert_ne!, sem suprimirgate. Suíte ampliada em execução.
- Suíte ampliadaFAIL1/62: CONNACKnegativo podia ocorrer antes de clientlerSUBACK;
  promptrev1.4antes sincronizarfixturepela Assinatura ativa, mantendoaceite.
- Suíte62PASS(33unit+29wire). ClippyFAILsemicolon_if_nothing_returned e
  too_many_lines102/100; promptrev1.5antessemicolon/helperRetry::next_before,
  sem suprimirwarnings, sem alterarbuddgets. Fullgates reexecutados.
- PASS finaldebug: fmt --all -- --check; cargo test --locked,63testes
  (34unit/0.36s+29wire/21.09s),0FAIL/0ignored; Clippyalltargets-Dwarnings;
  build --locked --release; license_inventory.py --check. Binário0.9.0 definido
  pelo coordenadorP92, sem novasdeps. Rust1.99.0(b940084d7)/Python3.14.4,
  Ubuntu26.04/WSL, diretórioacmebr-projects. Temposnão sãobenchmark.
- EnsaioisoladodebugPASS3: SIGINTentradainicial0.813s eSIGTERM0.106s,0CONNECT;
  idle90.004s com89PINGRESP,0retry. Sha256bináriodebugpré-helper:
  df1ae08af122242b2bf4cb5f16d760da0076463ae8286f83c816295fd5b93487.
  Essehashé preliminar, nãohashdoartefatofinal.
- PASS cargo test --locked --release:63testes (34unit/0.11s+29wire/12.93s),
  0FAIL/0ignored. Perfilrelease, não somentecompilação doartefato.
- PASS ensaiorelease final, [JSON](../../docs/evidence/client-c45-robustness.json):
  SIGINTentradainicial0.041s eSIGTERM0.013s,0CONNECT; idle90.001s com89PINGRESP,
  0retry; duração90.583s. Hash SHA256 doexecutável0.9.0:
  a7a89dba2245f56d4161d52a9767441257edca04be8042edcabf358e4bb05f58.
  Hashreconferidoapósensaio e igualaoarquivo atualtarget/release/mqtt-client.
  Esseé candidato local; exactclone/distrib/publicaçãoP92 ainda pelo coordenador.
- Interop mTLS/ACL/Mosquitto/brokerreal: campanhaP91 por outroagente; não declarar
  resultado aqui antes de receberevidência/coordenador.
- Commit/push/tag/release/deb/DockerHub: coordenador, ainda não feitosC45.

Implementação não adiciona deps nemestado local e não toca serviçosESP32/firewall/
credenciais reais. Alterações manifests/version são responsabilidade de release
coordenado. Não alterar tags existentes. Não encerrar fase3 somenteporfixtures.

Limites: LinuxWSLUbuntu26/Rust1.99 é ambiente dos gates; semMSRV1.88 exato,
Windowsnativo, arm64 ou promessaExactlyOnce através deQoS2recebidointerrompido.

## Contrato efetivo

--total-timeout-seconds1..86400/defaultNone; relógio único após primeiraentrada
e credenciais, sem renew portráfego/retry. Expiração sem DISCONNECT/graceextra,
portanto Willpodeocorrer e pubincertanãoreenvia. Sinalprioritário tambémtem
cleanup limitado ao totalrestante (promptrev1.6,unitnovo). I/OlocalTLSsíncrono
conta nobudget, masnãoépreemptivo/SLAtempo-real. Espera90srenova sóPUBLISH;
CONNACKreconectado mantémtimeoutprotocolo, nãoidle90. Retry0..10semreset e
jitter80..100%base100..5000ms (não criptográfico). QoS1duplicataslegítimas;
QoS2recebidointerrompido fail explícitorumqttc0.25.1.

As novas fixtures incluem4etapas total(CONNACK/SUBACK/PUBACK/aplicação),
tráfegoconstante,backoff/zeroattemptposterior,entradaposterior,CONNACKsilencioso
da segunda conexão, inválidosCLIsemrede/vazamento e QoS1duplicate2entregas.
SIGINT/SIGTERM reais em3estágios poll/backoff/entrada seguinte são6combinações;
fixtureslegadasauthsemretry/bytes/QoS2incerto e unitTLSlímitePEM também passaram;
não inferirinteroperabilidadeTLS/SAN/mTLS desseunit. Regressão
preexistenteC26budgetcontinua3conexõespara2retries mesmoapóssucesso.
