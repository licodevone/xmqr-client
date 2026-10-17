# C91 rev1 — client MQTT 5 com autenticação inicial SASL PLAIN/TLS

Entrada: client 0.24.0 usa rumqttc para CONNECT MQTT 5 e já recebe senha por arquivo privado ou prompt, sem colocá-la nos argumentos. Broker P159 definirá o método `PLAIN` com dados RFC 4616 em conexão TLS.

Objetivo: acrescentar opção explícita para autenticação inicial `PLAIN` apenas em MQTT 5 sobre TLS. Formar `Authentication Data` como `NUL authcid NUL passwd` com username e senha já validados; definir Authentication Method `PLAIN` no CONNECT; não duplicar as credenciais nos campos Username/Password convencionais. Rejeitar modos de transporte aberto, opções incoerentes e dados inválidos antes de conectar. Validar que CONNACK de sucesso confirma o mesmo método; não imprimir segredo em erro, JSONL ou log. Manter modo convencional por padrão.

Aceite: testes de parser CLI e wire sem segredos, sucesso/falha TLS real com broker P159, CONNACK sem método/método divergente, compatibilidade 3.1.1 e fluxo pub/sub; fmt/test/Clippy/build/licenças/CI do commit exato. Reautenticação via AUTH continua pendente; sem tag/release isolada até marco próprio.

Fontes normativas: [OASIS MQTT 5 §4.12](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html) e [RFC 4616 SASL PLAIN](https://www.rfc-editor.org/rfc/rfc4616).
