# Como usar o prompt do My Path (guia para quem não programa)

## 1. Antes de começar (uma vez só)
1. **Crie uma conta no GitHub** (https://github.com, botão *Sign up*, é gratuito). É lá que o código vai morar e onde os instaladores serão gerados automaticamente.
2. Escolha a ferramenta:
   - **z.ai, com GLM‑5.3‑Flash em modo Agent:** tem uma janela de contexto enorme (cerca de 1 milhão de tokens), então aguenta o prompt inteiro com folga. Se houver o modo `/goal`, use para tarefas longas.
   - **Arena.ai (Agent Mode):** roda num computador Linux Debian 13, o mesmo sistema da lista do projeto. Aqui dá para conectar o GitHub como app conectado.
   - Você pode alternar entre as duas: o projeto guarda a memória dentro do próprio repositório (pasta `docs/`).

## 2. Primeira sessão
1. Abra o arquivo **`PROMPT_MESTRE.md`**, copie **tudo** e cole na IA.
2. A IA vai:
   - confirmar que entendeu;
   - te guiar para criar o repositório;
   - montar a estrutura do projeto;
   - começar a **Fase 1** (resolver os limites técnicos).
3. Faça só o que ela pedir: clicar, baixar, testar e devolver o relatório.

## 3. Sessões seguintes (cole isto para continuar)
```
Continue o projeto My Path. Repositório: <cole o link do seu GitHub aqui>.
Leia docs/PROGRESSO.md, docs/DECISOES.md e docs/LIMITACOES.md,
me diga em 3 linhas onde paramos e siga o PROMPT MESTRE (eu não sei programar).
```
Se a IA "esquecer" as regras, cole o `PROMPT_MESTRE.md` de novo.

## 4. Como testar no seu computador
- A IA vai gerar o **My Path Probe**, um app de teste. Você baixa, abre, segue o roteiro na tela e ele cria um arquivo **`relatorio-probe-....html`**. Envie esse arquivo para a IA.
- **Para testar várias distros Linux sem formatar o PC:** use um pendrive com o **Ventoy** e coloque nele as ISOs do Debian 13, Ubuntu 26.04, Linux Mint 22.3 e Fedora 44. Dá para iniciar o PC por cada uma no modo "Live", sem instalar nada.
- **Touchpad só se testa num notebook de verdade.** Máquina virtual não serve para isso.
- Windows 7: se não tiver um PC com Win7, peça a alguém ou use uma máquina virtual (VirtualBox) só para ver se o app abre e grava cliques.

## 5. Avisos importantes
- **Windows 7:** o Tauri, que é a base do app, parou de suportar o Win7 em setembro de 2026. O projeto prevê uma versão "Legacy" separada e experimental. Se ela não der certo, existe um plano B (só gravação).
- **Licença:** MIT e Apache‑2.0 **permitem** uso comercial; nenhuma licença open source pode proibir. Por isso o código fica "MIT OR Apache‑2.0" (quem usa escolhe). Para impedir uso comercial de um **guia gravado**, o app oferece a licença **CC BY‑NC 4.0** no início da gravação. Se for vender algo, consulte um advogado.
- **Linux mais novo (Ubuntu 26.04 e Fedora 44) é só Wayland.** Ele é mais fechado por segurança, por isso o app terá uma tela "Preparar meu sistema", com um clique e talvez um pedido de senha.
- **Antivírus:** apps que "escutam" mouse e teclado podem ser confundidos com vírus. O projeto prevê assinatura digital gratuita para open source, para evitar isso.
