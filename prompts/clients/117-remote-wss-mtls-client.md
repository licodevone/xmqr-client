# C117 — destino WSS mTLS remoto com ativação explícita

Entrada: client 0.33.0 publicado com WSS mTLS limitado a loopback (C114/C115); perfil C116 aceito em `main` após a tag. Broker P214 preparado para listener remoto. Este prompt precede qualquer alteração funcional.

Objetivo: aceitar URL WSS com hostname/IP não loopback apenas mediante flag explícita própria, mantendo validação completa de CA/SAN, certificado cliente, credenciais MQTT e subprotocolo `mqtt`. Sem fallback para WS, TLS-senha ou HTTP; WS aberto e TCP atuais inalterados.

Escopo: parser/transportes MQTT 3.1.1/5, perfil TOML, documentação, fixtures e integração Docker descartável com P214. URL sem usuário/senha, query, fragmento ou caminho vazio. Flag sem URL WSS remoto deve falhar; URL remota sem flag deve falhar; não inserir segredo na linha de comando. Estado durável por URL e navegador permanecem fora do recorte.

Aceite: pub/sub 3.1.1/5 por hostname remoto com SAN correto e ACL, negativos de flag, CA/SAN, certificado, senha, ACL, Origin, URL e falha de rede sem downgrade; regressão WSS local, WS `open-lab` e TCP/TLS; fmt, suíte, Clippy, build, licenças/distribuição e CI da fonte exata. Registrar parcial da fase 7, sem tag/release isolada.
