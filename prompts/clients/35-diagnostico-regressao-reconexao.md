# C35 rev1 — diagnóstico e regressão de reconexão

Entrada: main 9619776, limpa; client 0.5.0 e broker vizinho 0.10.0.
Objetivo: diagnósticos estáveis sem conteúdo sensível e teste de reconexão
compatível com entrega pelo menos uma vez do QoS1.

Escopo: src/session.rs, validation/C27-integration.py, README e registro C35.
Testar sessões clean/persistent, receber before e after sob deadline, aceitar
somente duplicatas dos payloads esperados, encerrar por SIGINT e validar exit0.
Não ampliar quotas, alterar retry/TLS/SAN ou persistência, registrar secrets,
editar firmware, usar estado real ou iniciar serviço permanente.

Aceite: mensagens de erro não formatam pacotes/certificados/credenciais; testes
de classificação e integração, fmt/test/clippy/build no Ubuntu26. Confirmar
QoS1 em OASIS3.1.1 §4.3.2 antes de alterar asserções. Sem promessa de entrega
ao consumidor de negócio. Telemetria ESP32 por comando documentado, sem leitura
automática dos payloads reais. Commit/push já autorizados; sem tag/release.
