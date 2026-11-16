# C158 v1.0 — instaladores Client 0.44.0

Base `46f96b51336b2416c35f47dfb130510b0eef2e54`. O C157 consolida o
comando `bench` no commit de validação. Os registros editados localmente
ficam fora das fontes empacotadas.

- Usar o builder Debian deste repositório para Ubuntu 26.04 amd64.
  O pacote deve conter `mqtt-client` 0.44.0 e `mqtt-bridge` 0.5.0.
- Usar o builder WiX do Broker e o manifesto de serviço deste Client
  para o MSI Windows x64. O MSI Client instala apenas `mqtt-client`.
- Preservar perfis, opções de TLS e regras de proteção dos arquivos.
  Conferir versão, `bench --help`, probe de serviço, inventário e SHA-256.
  A instalação nativa só pode ser exercitada em ambiente descartável.
- Registrar resultados e bloqueios em `prompts/registros/C158-instaladores-044.md`.
  Não reutilizar executáveis de outra versão nem criar tag sem aceite.
