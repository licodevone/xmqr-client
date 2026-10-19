# C145 v1.0 — preservar builds locais fora do Git

Data: 2026-10-19. Entrada: `main`/origin `25c2e8a`, client 0.42.0,
bridge 0.4.0. CI da fonte `39194165930` PASS. C144 foi incorporado nesse
commit junto com 1.861 arquivos gerados em `vendor/rumqttc-0.25.1/target/`.
O diretório preexistia como saída não rastreada e deve continuar no disco.

Objetivo: retirar somente esses artefatos do índice e impedir sua inclusão
novamente usando a regra `**/target/`. Não apagar arquivos locais, reescrever
histórico, alterar runtime, dependências, versão, pacote ou tags existentes.

Escopo: `.gitignore`, índice Git, registro C145 e reconciliação do registro
C144 com a CI efetivamente executada. A evidência histórica local de C144
permanece identificada separadamente da CI posterior.

Aceite: contagem rastreada sob o target vendorizado igual a zero; diretório e
arquivos locais preservados; hashes de arquivos locais antes/depois idênticos;
diff funcional vazio fora dos documentos e regra de exclusão. `git diff --check`,
commit/push e verificação do SHA remoto. Sem repetir testes de protocolo para
uma remoção exclusiva de artefatos gerados do índice.
