# C95 rev1 — Message Expiry Interval no Will MQTT 5

Entrada: C94 oferece metadados selecionados do Will, enquanto `LastWillProperties` da dependência fixada também contém `message_expiry_interval`. Este prompt precede o código.

Objetivo: adicionar `--will-message-expiry-seconds 0..4294967295` para Will MQTT 5, combinável com Will Delay e demais propriedades. Validar opção única, protocolo 5 e Will completo antes da rede; não confundir com `--message-expiry-seconds` de PUBLISH. Enviar `0x02` no CONNECT e preservar comportamento legado sem a opção.

Aceite: parser e bytes do CONNECT, limites/duplicata/truncamento de argumento, integração com broker P164 comprovando que prazo inicia na publicação efetiva, regressão C94/MQTT 3.1.1, fmt/Rust/Clippy/release/licenças/distribuição/CI do commit exato. Atualizar documentação e checklist honestamente; não etiquetar isoladamente.

Fonte: [MQTT 5 §3.1.3.2.4](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html).
