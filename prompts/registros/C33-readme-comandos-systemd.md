# C33 rev1 — execução

README detalha controle e início automático da assinatura, publicação oneshot,
logs, código de saída, instâncias, ClientIDs, configurações, falhas e WSL.
Units conferidas: publicação sem Install/reinício; assinatura sem ExecReload.
Links locais, bash -n dos oito blocos shell e git diff --check PASS.
Código, versões e credenciais preservados; nenhum serviço real controlado para
validar texto.

Compatibilidade Ubuntu24.04.5 amd64/WSL2, glibc2.39: binários publicados copiados
do Ubuntu26. C27: três PASS e um FAIL por mensagem before repetida após restart;
na repetição quatro PASS. Três testes de shutdown do broker PASS. Causa da
variação não confirmada. Instalação deb e units Ubuntu24 NOT_RUN; não declarar
suporte integral. Não instalar pacotes nem modificar serviços do usuário.

Commit/push autorizados; sem tag ou release.
