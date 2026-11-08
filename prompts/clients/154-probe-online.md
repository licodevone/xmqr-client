# C154 v1.2 — diagnóstico MQTT online

## Entrada e objetivo

Branch `codex/c153-check-config`, limpa, base
`04189b6124aca075dd944b937351eb87380cc47c`. O Client local 0.43.0 valida
configuração sem rede. Adicionar `mqtt-client probe` para localizar falhas
no caminho até o broker em MQTT 3.1.1 e 5.

## Escopo e contrato

- Entrada e ajuda em `src/main.rs`; diagnóstico em `src/probe.rs`.
  Reutilizar configuração de sessão e TLS de `src/session.rs` e
  `src/v5_session.rs`. Usar parser, perfil e leitor privado de credenciais
  existentes. Atualizar README, matriz de paridade e testes de CLI/wire.
- Por padrão, verificar CONNECT/CONNACK. `--topic` acrescenta uma assinatura
  exata. `--round-trip` exige `--topic` e publica uma única mensagem de
  diagnóstico QoS 0 sem retenção após SUBACK. Só aceitar a recepção no tópico
  exato, com marcador efêmero correspondente e sem flag RETAIN.
- Uma sessão limpa com identificador efêmero dedicado, sem reconexão, Will,
  outbox, journal ou alteração de perfil. Continuar validando CA/SAN, mTLS e
  opt-in de WSS remoto. Laboratório aberto somente em loopback. Escopo inicial
  cobre credencial MQTT regular. Não migra dados existentes.
- Um prazo monotônico cobre as fases online. JSON compacto de schema 1 e
  código 0/1/2 para sucesso/falha online/entrada inválida. Diagnósticos usam
  categorias fixas e omitem material privado, payload e texto remoto.
  Só atribuir TCP/TLS/WebSocket/autenticação quando houver evidência tipada.

## Aceite e validação

Verificar opções inválidas, recusa de conexão e de assinatura, mensagem alheia,
sucesso nas duas versões e limite de espera. Rodar fmt, build, Clippy, testes
portáveis e fixtures em broker separado. Registrar PASS, FAIL e BLOCKED em
`prompts/registros/C154-probe-online.md`. A promoção exige a CI do commit exato
e a validação dos pacotes da versão. `bench` recebe prompt próprio.

### Revisão 1.1

O build padrão parou por dependência ausente no cache e DNS do crates.io.
A compilação direta com os artefatos C151 revelou colisão do nome `Outgoing`
na adaptação de eventos. Qualificar o tipo do pacote, repetir a compilação
e executar as fixtures diretamente quando a compilação via Cargo estiver
indisponível. O resultado direto não substitui o gate Cargo/CI.

### Revisão 1.2

O Clippy direto com os lints pedantic do manifesto detectou imports
curinga de variantes. Qualificar os enums e repetir o Clippy com
`-D warnings -D clippy::all -W clippy::pedantic -F unsafe_code`.
