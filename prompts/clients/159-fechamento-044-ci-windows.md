# C159 v1.0 — validação Windows do Client 0.44.0

Entrada: `46f96b5`, registros C155/C157/C158 locais preservados. A 0.44.0
contém probe, check-config e bench, com pacote e aceite de rede pendentes.

Adicionar `.github/workflows/windows-release.yml` para compilar e testar
no runner Windows hospedado, com toolchain MSVC e dependências locked.
Executar a suíte CLI/wire normal, o manifesto independente de serviço,
seus testes e o probe do adaptador. Uploadar binário e evidências de teste
sem publicar credenciais. O MSI continua usando o template do Broker.

O workflow P284 do Broker compila o par fixado por SHA e exercita o helper
SCM do Client contra o Broker instalado, em VM com configuração privada.

Manter a CI Ubuntu, os testes MQTT/bridge e as versões independentes.
Validar YAML e caminhos localmente, enviar a branch de validação quando
possível e acompanhar os jobs. Registrar resultados reais em C159.
Tag e release 0.44.0 dependem dos gates de pacote, rede e serviço.
