# C149B v1.0 — exportação da fonte aceita no checkout compartilhado

Data2026-10-19. Fonte congelada9e1d011, CI39301735099 com seis jobsPASS,
imagem final e pacote já validados. Exportador recusou árvore suja no WSL
antes de gravar os assets: GitWindows reporta limpa com core.autocrlf=true;
GitWSL sem essa configuração vê61arquivos somente por finaisCRLF/LF.
Não existe falha de produto, pacote ou imagem inferida dessa recusa.

Antes de repetir, comprovar diff canônico vazio usando GitWSL com a mesma
configuração de conversão do GitWindows, core.autocrlf=true por processo.
Repassar essa configuração temporária ao exportador via GIT_CONFIG_COUNT/
KEY_0/VALUE_0. O gate gitdiffHEAD permanece ativo; diferença real interrompe
exportação. Não modificar arquivos, índice, configuração Git persistente,
histórico ou hash da fonte aceita. Este prompt e registro entram no commit
documental posterior; não mudam a revisão de runtime9e1d011.

Aceite: GitWindows limpo, GitWSL normalizado sem diferença rastreada, mesmos
HEAD/CI/labels/ELFs; exportação limpa e seis assets novos com hashes. Preservar
recusa original no registroC149. Não refazer gates funcionais por conversão
de finais de linha que não muda a fonte canônica; publicação continua exigindo
tag/digest e seis downloads conferidos conformeC149.
