# C160 v1.0 — Corrigir os gates do Client 0.44.0

Entrada: CI 45705105570 do commit 4a68270. Dois testes TCP loopback falharam
quando o servidor de teste fecha imediatamente após sua última resposta.
A imagem também falha ao coletar o aviso MIT de windows-permissions 0.2.4.

Manter a conexão da fixture até o encerramento pelo cliente. Preservar
as asserções existentes. Incluir o aviso MIT revisado pelo Broker, com
proveniência, hashes e verificação dos manifestos contra o crate resolvido.
Sem alterações de protocolo, versão ou dependências. Aceite pela CI Linux
e Windows e pela coleta de avisos do pacote e da imagem.

Revisão v1.1: a nova CI aprovou as fixtures e apontou três erros de Clippy 1.100. Usar assert_eq nas asserções de ausência de saída e alocar a future grande de run_cli na heap, mantendo o diagnóstico estrito.
