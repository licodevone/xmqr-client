# C43 rev1.5 — client na validação externa da fase 2

Pedido vigente: finalizar fase2 de ambos os projetos, prompt-first, commits/push
e marcos de release somente aceitos. Entrada2026-10-13 client0.8.1
HEADbc94bd04d36f2bf5e61f3d60391bae142d9407b6; broker/admin0.11.3
HEAD4e199b2244cc7758414531425dc1f1f483ac40ba. Duas archivesclient0.7.1
não rastreadas pertencem ao usuário e não entram no commit.

Objetivo: expandir interop 3.1.1 externa e separar checklist client/broker/ESP32.
Escopo: scripts/verify_mosquitto_phase2.py, scripts/test_mosquitto_phase2.py,
docs/phase2-acceptance.md, docs/mosquitto-parity.md, README e registroC43.
Client CLI não vira gerador de carga; nenhum Rust, dependência ou versão muda
se a validação não demonstrar defeito. Defeito exige revisar prompt antes de fix.

Docker controlado pelo WindowsCLI; imagens públicas exatas broker0.11.3,
client0.8.1 e Mosquitto2.0.22 já disponíveis, pin de imageID/digest registrado.
CA e certificados de1dia recém-gerados em área temporária privada,
senha conhecida de fixture; nenhuma credencial de instalação real.
Rede container namespace loopback, sem portas host publicadas; processos,
containers/volumes exclusivamente de teste com timeout e cleanup delimitados.

Matriz QoS0/1/2: XMQRclient↔Mosquitto pub/sub e mosquitto_pub/sub↔XMQRbroker,
mTLS+senha+ACL, retained replay/update/delete, saída JSONL e bytes íntegros;
assinante pronto pelo SUBACK/sinal da CLI e não apenas sleep. Negativos:
CA/SAN/senha inválidos e ACL negada; SUBACK3.1.1 não informa causa específica,
PUBACK/PUBCOMP não provam processamento de negócio. GateESP32 não executado
por Docker/fixture. OASIS3.1.1 §§3.3.1.3,3.3.4,3.8.4,4.3 confirmados em
https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html .

Aceite: contratosPython negativos, cenários externos reais, versão e hashes,
gatesfmt/test/clippy/build/inventário client, resultados PASS/FAIL/BLOCKED/NOT_RUN
sem extrapolar conformidade MQTT5/produção. Arquivos de resultado não guardam
password_file/cert/key/payload/log sensível. Publicar somente evidências sanitizadas.
Compatibilidade: nenhuma. Commit/push autorizados; sem nova versão/tag binária
apenas por harness. Não avançar fase3 se capacidade ouESP32 não aceitos.

## rev1.1 — diagnóstico da readiness externa

Primeira execução Docker falhou no prazo do SUBACK/assinatura; cleanup executado,
nenhum PASS declarado. Registrar kind/exit code e classes de erro redigidas
para identificar causa, sem imprimir argumentos de senha ou dumps de fixtures.

## rev1.2 — ClientID explícito da referência

Diagnóstico: Mosquitto sub saiu7 (conexão perdida), sem falhaTLS/senha/opção.
Harness antigo do broker sempre passa -i; C43 não passava. Broker tem contrato
de ClientID não vazio e não faz atribuição automática. Definir -i único/bounded
para pub/sub Mosquitto. Não ampliar contrato CONNECT para encobrir o harness.

## rev1.3 — ampliar contrato externo antes das novas fixtures

Acrescentar Will após SIGKILL de container próprio, sessão persistente offline
após DISCONNECT e servidores próprios com certificado da CA válida mas SAN
errado. Executar nas duas direções nos brokersXMQR/Mosquitto, com ClientIDs
fixados nas retomadas, prazo finito e nenhum estado da instalação real.

## rev1.4 — limite do ClientID persistente

Matriz base47 PASS. Ampliação falhou na CLI XMQR exit2 antes da conexão:
ID c43-session +12hex tem24bytes; contratoCLI máximo23. Reduzir prefixo
da fixture e acrescentar teste do tamanho/alfabeto; não ampliar quota da CLI.

## rev1.5 — verificação documental

59cenáriosPASS. Link-checkPowerShell falhou por diretório relativo vazio no
README antes de commit; repetir usando caminho absoluto do documento.
Não alterar links/contratos apenas para contornar validador.
