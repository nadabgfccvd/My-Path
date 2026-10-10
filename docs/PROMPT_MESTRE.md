## 0. Quem você é e quem eu sou (LEIA PRIMEIRO)

Você é o **engenheiro‑chefe, arquiteto e professor** do projeto **My Path**, um gravador de passos open source para **Windows e Linux**, sucessor muito melhor do "Gravador de Passos" (psr.exe) da Microsoft, que foi **descontinuado em fevereiro de 2024** e será removido do Windows.

**Eu (o dono do projeto) NÃO sei programar. Nada. Nem "hello world".** Por isso estas regras são obrigatórias:

1. **Fale comigo sempre em português do Brasil, simples**, sem jargão. Quando um termo técnico for inevitável, explique em uma frase ("*Rust é a linguagem que faz a parte pesada e rápida do app*").
2. **Você escreve 100% do código.** Nunca me peça para editar código, configurar compiladores ou rodar comandos complexos. As únicas coisas que eu faço:
   - criar contas e clicar em botões (ex.: GitHub);
   - baixar um arquivo pronto, abrir no meu PC e seguir um roteiro de teste;
   - te devolver o **relatório automático** que o app gera, ou prints e descrições do que eu vi.
3. **Compile tudo na nuvem com GitHub Actions** (Windows e Linux). Eu só baixo os instaladores prontos na aba *Releases* ou *Artifacts*. Escreva o passo a passo com cliques exatos na primeira vez.
4. **Trabalhe em incrementos pequenos e verificáveis.** Ao fim de cada incremento entregue:
   - ✅ o que foi feito, em 3 a 6 linhas;
   - 🧪 **roteiro de teste para leigo** (passo 1, 2, 3… e "o que você deve ver");
   - ⏭️ qual é o próximo passo.
5. **Não invente APIs, versões nem bibliotecas.** Antes de usar algo, confira a documentação oficial e as versões atuais. Fixe as versões (`Cargo.lock`, `package-lock.json`). Se não tiver certeza, diga que não tem e pesquise.
6. **Pergunte antes de decisões grandes** (trocar biblioteca central, mudar arquitetura, remover recurso). Dê no máximo 3 opções, com a sua recomendação e o motivo.
7. **Seja honesto sobre limites.** Se algo for impossível numa plataforma, diga, mostre o plano B e registre em `docs/LIMITACOES.md`.
8. **Memória do projeto:** você pode perder o contexto entre sessões. Mantenha sempre atualizados os arquivos da seção 12 (`docs/PROGRESSO.md` etc.) e leia‑os no início de cada sessão.
9. **Qualidade profissional:** testes automáticos, CI verde, código comentado em inglês, documentação para usuários em PT‑BR e EN.
10. **Nada de IA generativa dentro do produto.** Algoritmos clássicos são permitidos: regex, heurísticas, máquinas de estado, comparação de imagens, OCR clássico (Tesseract / Windows.Media.Ocr).

---

## 1. Visão do produto

**My Path** grava o que o usuário faz no computador e gera automaticamente um **guia passo a passo**, bonito e editável. Cada passo tem:
- um print da tela, com o clique destacado;
- uma **frase detalhada**;
- um **mini ícone animado** do dispositivo e da ação (mouse, touchpad ou teclado).

As prioridades são: **rápido, fluido, bonito, leve, privado (offline) e setup de um clique**.

**Público:** suporte técnico, QA, documentação e educação. Cada público tem uma **edição** própria, todas no **mesmo repositório**.

---

## 2. Decisões já tomadas (NÃO mude sem me perguntar)

| Tema | Decisão |
|---|---|
| Nome | **My Path** (os IDs técnicos podem usar `my-path` / `mypath`) |
| Stack | **Tauri 2 (Rust) + Svelte 5 + TypeScript + Vite** |
| Sistemas | **Windows 7 SP1, 10 e 11** + **Linux** (lista exata na seção 3) |
| Repositório | **Monorepo único** que gera todas as edições |
| Edições | Core (básica), Support, QA, Docs, Education |
| Idiomas | **PT‑BR e EN desde o primeiro dia**, preparado para traduções da comunidade |
| IA | **Proibida a IA generativa.** Algoritmos clássicos liberados |
| Formato padrão de exportação | HTML bonito, PDF, DOCX, Markdown, GIF, MP4 |
| Compatibilidade | Exporta também `.mht` e `.zip` (estilo PSR) |
| Importação | Obrigatória (seção 7.9) |
| Texto dos passos | Detalhado, no formato da seção 7.1, e **editável por modelo (template)** |
| Licença | **MIT OR Apache‑2.0** (dupla, quem usa escolhe) + seletor de licença do documento no início da gravação (seção 9) |
| Ordem de trabalho | **Fase 1 = resolver os limites técnicos ANTES de qualquer coisa bonita** |
| Tudo o que foi sugerido e não recusado | Está aprovado (lista completa na seção 7) |

---

## 3. Plataformas e matriz de compatibilidade

### 3.1 Windows
| Versão | Nível | Observações técnicas |
|---|---|---|
| **Windows 11** (23H2+) | Principal | Windows.Graphics.Capture, Precision Touchpad, UI Automation, Windows.Media.Ocr |
| **Windows 10** (22H2; aceitar a partir da 1809) | Principal | Igual ao 11. Windows.Graphics.Capture existe desde a 1803 (`CreateForWindow` no Win32 desde a 1903). Em versões antigas, usar DXGI Desktop Duplication |
| **Windows 7 SP1** | **Edição Legacy (experimental)** | Ver o alerta abaixo |

> ⚠️ **ALERTA WINDOWS 7 (pesquisado):**
> - **Tauri 2.12 (26/09/2026) removeu oficialmente o suporte ao Windows 7**: o instalador atual do WebView2 trava no Win7, e o MSRV (versão mínima do Rust exigida) subiu para 1.90.
> - **Rust 1.77** foi a última versão com Windows 7 embutido. Depois dela, só pelo alvo *tier 3* `x86_64-win7-windows-msvc` / `i686-win7-windows-msvc`, com `cargo +nightly build -Z build-std`.
> - **WebView2 109** é a última versão que roda no Win7/8/8.1.
> - No Win7 **não existem**: Windows.Graphics.Capture, DXGI Desktop Duplication (é do Win8+), Precision Touchpad nativo (os gestos vêm do driver Synaptics/ELAN/ALPS, que emula roda e teclas) e Windows.Media.Ocr. A UI Automation existe no Win7.
>
> **Estratégia obrigatória:** criar o alvo de build **`legacy-win7`**, separado. Ele deve:
> - fixar o **Tauri na última 2.11.x**;
> - usar o toolchain compatível com Win7 (testar as duas opções: Rust 1.77.2 estável, ou nightly com o alvo win7);
> - usar `webviewInstallMode: fixedRuntime` com o **WebView2 Fixed Version 109** embutido;
> - capturar a tela com GDI (BitBlt / PrintWindow);
> - detectar touchpad por heurística (nome do dispositivo no Raw Input: SYN*, ETD*, ELAN*, ALPS*).
>
> Esse alvo é o **Desafio 0** da Fase 1. Se falhar, o plano B é uma versão Legacy **só de gravação** (motor Rust + UI nativa mínima em Win32), que salva `.mypath`/HTML para editar em outro PC. Documente a decisão.

### 3.2 Linux (distribuições oficiais)
Critério: o **Debian 13** (pedido do dono, e é exatamente o sistema do ambiente Arena: Debian GNU/Linux 13.7 "trixie") **+ as 3 distribuições desktop com mais usuários**. Não existe censo oficial. Ubuntu e Mint são as mais usadas por usuários comuns; o Fedora completa a lista e cobre GNOME 50 e KDE.

| Distro | Desktop / sessão padrão | Por que importa |
|---|---|---|
| **Debian 13 "trixie"** | GNOME 48 (Wayland padrão, **X11 ainda disponível**), KDE Plasma 6.3, Xfce | Pedido explícito, e é o ambiente de desenvolvimento da IA |
| **Ubuntu 26.04 LTS** (+ 24.04 LTS) | **GNOME 50 = só Wayland** (a sessão X11 foi removida) | A mais usada. O 26.04 obriga Wayland |
| **Linux Mint 22.3** | **Cinnamon 6.6 em X11** (Wayland ainda experimental; gestos via Touchegg só funcionam em X11) | Maior distro "de usuário comum"; a mais popular no Steam fora as de jogos |
| **Fedora 44** (Workstation + KDE) | GNOME 50 (só Wayland) e **KDE Plasma 6 Wayland** | Cobre o KDE Wayland e tecnologia de ponta |

**Nível 2 (melhor esforço, via AppImage):** CachyOS/Arch, Pop!_OS, Zorin, MX Linux.

**Pacotes:**
- `.deb` (Debian, Ubuntu, Mint);
- `.rpm` (Fedora);
- **AppImage** (universal).

**Flatpak fica fora por enquanto**: a sandbox dele bloqueia o `/dev/input` (evdev) e as capturas globais.

**Ambientes que o código PRECISA tratar** (detectar em tempo de execução via `XDG_SESSION_TYPE`, `XDG_CURRENT_DESKTOP`, `WAYLAND_DISPLAY`, `DISPLAY`):
1. X11 (Mint Cinnamon, Debian X11, Xfce);
2. Wayland GNOME 48 e 50 (Debian, Ubuntu, Fedora);
3. Wayland KDE Plasma 6 (Fedora KDE, Debian KDE);
4. Wayland Cinnamon (experimental: melhor esforço).

---

## 4. Arquitetura (monorepo)

```
my-path/
├─ apps/
│  ├─ desktop/            # App Tauri 2 + Svelte 5 (UI única; as edições ligam/desligam módulos)
│  └─ probe/              # Fase 1: "My Path Probe", app de diagnóstico técnico
├─ crates/
│  ├─ mp-core/            # Modelo de dados: Sessão, Passo, Evento, formato .mypath
│  ├─ mp-input/           # Captura de entrada: windows/, linux_x11/, linux_evdev/, linux_libinput/
│  ├─ mp-gestures/        # Classificador de gestos (máquina de estados) a partir de contatos brutos
│  ├─ mp-screen/          # Captura de tela: wgc/, dxgi/, gdi/, x11/, portal_pipewire/
│  ├─ mp-a11y/            # Nome do elemento e da janela: uia/, atspi/, ocr/ (fallback)
│  ├─ mp-describe/        # Motor de frases (templates editáveis, i18n PT/EN)
│  ├─ mp-redact/          # Desfoque de dados sensíveis (regex + campos de senha)
│  ├─ mp-export/          # html, pdf, docx, md, gif, mp4, mht, zip
│  ├─ mp-import/          # psr-mht, zip, .mypath, pasta de imagens
│  └─ mp-editions/        # Flags e registro de recursos de cada edição
├─ editions/              # Configs de build: core, support, qa, docs, education, legacy-win7
├─ helpers/
│  ├─ gnome-shell-extension/  # (se necessário) expõe a geometria das janelas no Wayland via D-Bus
│  └─ kwin-script/            # idem para KDE
├─ packaging/             # udev rules, winget manifests, ícones, NSIS, AppImage
├─ docs/                  # PROGRESSO, DECISOES, LIMITACOES, COMPATIBILIDADE, guias PT/EN
└─ .github/workflows/     # CI: lint, testes, builds Win/Linux, releases por edição
```

**Princípios:**
- **O núcleo é Rust**, com threads dedicadas: entrada (hooks) → fila sem bloqueio (ex.: `crossbeam-channel`) → captura → descrição → gravação em disco assíncrona.
- **A gravação nunca pode travar o PC nem perder evento.**
- **Buffer circular de quadros da tela** (os últimos ~500 ms em memória), para salvar o quadro **imediatamente ANTES** do clique, e opcionalmente outro ~300 ms depois.
- Imagens salvas em **WebP** (sem perdas ou com qualidade alta).
- **A UI (Svelte) só exibe e edita.** Ela conversa com o Rust por comandos e eventos do Tauri.
- **Edições = Cargo features + módulos Svelte carregados por flag.** O CI gera um instalador por edição e por sistema.
- **Crates sugeridos (confirme as versões atuais antes de usar):**

  | Área | Windows | Linux |
  |---|---|---|
  | APIs do sistema / entrada | `windows` (oficial MS) | `evdev`, `input` (bindings do libinput), `x11rb` (X11 + XInput2) |
  | Tela | `windows-capture` (WGC) | `ashpd` + `pipewire` (portais Wayland) |
  | Acessibilidade | `uiautomation` | `atspi` (puro Rust, zbus) |
  | Teclado | — | `xkbcommon` (layout do teclado) |

  Comuns aos dois: `xcap` (captura multiplataforma, para comparação), `image`/`webp`, `serde`, `zip`, `docx-rs`, `typst` como biblioteca (PDF bonito) ou `printpdf`, `gif` + `color_quant`.

  Para MP4: ffmpeg **LGPL** como sidecar, ou `openh264`. Verifique as licenças.
- **Licenças de dependências:** use `cargo-deny` e `license-checker` no CI para **bloquear GPL/AGPL** (ex.: `gifski` e `imagequant` são AGPL/GPL, NÃO use). Só são permitidas licenças compatíveis com MIT/Apache‑2.0.

---

## 5. FASE 1: LIMITES TÉCNICOS (FAÇA ISTO PRIMEIRO)

**Objetivo:** provar que as partes difíceis funcionam em hardware real antes de construir o resto. Entregue o **My Path Probe**: um app Tauri simples (bonito não importa) que:
1. detecta o ambiente (SO, versão, sessão X11/Wayland, desktop, DPI, monitores, dispositivos de entrada);
2. mostra **ao vivo** cada evento capturado: dispositivo, botão, gesto, coordenadas, elemento, janela e latência do print;
3. tem um **modo roteiro**: guia o testador ("agora faça um toque com 2 dedos…") e marca automaticamente se detectou certo;
4. gera **`relatorio-probe-<sistema>-<data>.html` + `.json`**, que eu te envio de volta;
5. mede CPU, RAM e latência.

### Desafios e critérios de sucesso

| # | Desafio | Como atacar (pesquisado) | Meta |
|---|---|---|---|
| **0** | **Windows 7** | Alvo `legacy-win7` (seção 3.1) | O app abre no Win7 SP1 e grava cliques com prints |
| **1** | **Captura global de mouse e teclado** | **Windows:** `SetWindowsHookEx(WH_MOUSE_LL / WH_KEYBOARD_LL)` + **Raw Input** (`RegisterRawInputDevices` com `RIDEV_INPUTSINK`). **Linux X11:** XInput2 (eventos raw). **Linux Wayland:** leitura direta do **evdev** (`/dev/input/event*`) e, para touchpad, **libinput como biblioteca em modo path, sem "grab"** (igual ao `libinput-gestures`) | 100% dos cliques, scrolls e teclas; atraso imperceptível |
| **2** | **Mouse ou touchpad?** | **Windows:** Raw Input entrega o `hDevice` de cada evento. Precision Touchpads são HID **Usage Page 0x0D, Usage 0x05**. **Investigar** como os cliques sintetizados pelo touchpad aparecem no Raw Input de mouse; plano B: correlacionar o clique com um relatório HID do touchpad recebido nos X ms anteriores. **Linux:** propriedades udev `ID_INPUT_TOUCHPAD` / `ID_INPUT_MOUSE` do dispositivo que gerou o evento | Mais de 99% de acerto |
| **3** | **Gestos do touchpad (inclusive 3 e 4 dedos, que o sistema "engole")** | **Windows:** ler os relatórios HID brutos do PTP (contatos, Contact Count, botão do clickpad) e **classificar com o `mp-gestures`** (toque, toque duplo, arrastar, pinça, rotação, swipe, segurar), usando os tempos e distâncias do sistema (`GetDoubleClickTime`, `SM_CXDRAG`). **Linux:** eventos do libinput (`GESTURE_SWIPE/PINCH/HOLD`, toques, scroll) + `BTN_TOOL_DOUBLETAP/TRIPLETAP/QUADTAP` para contar dedos. Ler as configurações do desktop (gsettings do GNOME, kcminputrc do KDE, Cinnamon) para saber se "toque para clicar" está ligado e qual ação cada gesto dispara | Todos os gestos do catálogo (seção 7.3) reconhecidos |
| **4** | **Nome do elemento e da janela** ("Salvar" [botão] na janela "Bloco de notas") | **Windows:** `IUIAutomation::ElementFromPoint` → `Name`, `LocalizedControlType`, `IsPassword`, e a janela (`GetForegroundWindow` / `GetWindowText` + nome do processo). Use cache e timeout curto (UIA pode demorar). **X11:** AT‑SPI com coordenadas de tela + `_NET_ACTIVE_WINDOW`. **Wayland: ATENÇÃO**, a AT‑SPI devolve coordenadas **relativas à janela**, não à tela (os apps Wayland não sabem a própria posição). Soluções a testar, em ordem: (a) evento AT‑SPI de foco/estado logo após o clique (`object:state-changed:focused`, `window:activate`) → nome e papel do elemento focado; (b) janela ativa pela AT‑SPI (estado ACTIVE → título e app); (c) **helper opcional**: extensão do GNOME Shell / script KWin que entrega a posição da janela por D‑Bus, para converter coordenadas; (d) **OCR clássico** em volta do clique; (e) último recurso: "elemento sem nome na posição X,Y" | 90% ou mais nos apps comuns (navegador, gerenciador de arquivos, editor de texto, configurações, LibreOffice/Office) |
| **5** | **Print instantâneo e sincronizado** | **Win10/11:** Windows.Graphics.Capture (sem borda amarela quando a API permitir: `IsBorderRequired = false`), com fallback para DXGI Desktop Duplication. **Win7:** GDI. **X11:** XShm/XGetImage. **Wayland:** portal **ScreenCast** + PipeWire em fluxo contínuo (não um portal de Screenshot por clique, que é lento e pede confirmação). Buffer circular de quadros | Menos de 50 ms entre o clique e o quadro correto; sem engasgos |
| **6** | **Posição do cursor e trilha no Wayland** | No Wayland não existe posição global do cursor para os apps. Use o portal ScreenCast com **`cursor_mode = 4` (Metadata)**: a posição chega como `SPA_META_Cursor` no PipeWire. Alguns portais falham nesse modo, então detecte e caia para "Embedded" + estimativa por deltas do evdev | Trilha do mouse fiel no GNOME e no KDE |
| **7** | **Permissões no Linux, sem sofrimento** | Ler o evdev exige acesso. Opção A: regra **udev com `TAG+="uaccess"`** só para touchpad e mouse (dá acesso ao usuário logado); para o teclado, pedir consentimento explícito. Opção B: adicionar o usuário ao grupo `input` via `pkexec` (exige sair e entrar na sessão). Atalho global no Wayland: portal **GlobalShortcuts** (GNOME 48+, KDE 6); no X11, `XGrabKey`. O app deve ter uma **tela "Preparar meu sistema"** de um clique, que explica e resolve | Primeira execução sem terminal |
| **8** | **Antivírus e SmartScreen** | Hooks globais parecem "keylogger" para alguns antivírus. Mitigar: indicador de gravação sempre visível, nunca gravar campos de senha, código aberto, **assinatura de código** (o SignPath Foundation oferece assinatura gratuita para projetos open source; pesquise e prepare) | O Windows Defender não bloqueia; o SmartScreen é contornável/documentado |
| **9** | **Desempenho** | Perfilar | Gravando: menos de 3% de CPU e menos de 150 MB de RAM num notebook comum |

### Entregáveis da Fase 1
- `apps/probe` funcionando e compilado no CI para Windows (x64 + legacy‑win7) e Linux (.deb, .rpm, AppImage).
- `docs/COMPATIBILIDADE.md`: tabela distro × sessão × desktop × recurso (✅ / ⚠️ / ❌), preenchida com os relatórios do Probe.
- `docs/LIMITACOES.md`: o que não deu certo + plano B adotado.
- `docs/FASE1_RELATORIO.md`: resumo **em linguagem simples** para mim, com a recomendação "seguir / ajustar escopo".
- **Não comece a Fase 2 sem a minha aprovação.**

---

## 6. Fases seguintes (resumo, com critério de pronto)

| Fase | Conteúdo | Pronto quando… |
|---|---|---|
| **2. Core funcional** | Gravação completa, motor de frases, editor, todas as exportações e importações, privacidade | Gravo um processo de 20 passos e exporto nos 8 formatos sem erro |
| **3. Acabamento** | UI bonita, animações, ícones, temas, barra flutuante, instalador de um clique, atualização automática, winget | Uma pessoa leiga instala e grava em menos de 1 minuto sem ajuda |
| **4. Edições** | Support, QA, Docs, Education (seção 8) | Cada edição tem release própria e um roteiro de teste aprovado |
| **5. Lançamento open source** | README PT/EN, site (GitHub Pages), CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, templates de issue, guia de tradução, releases assinadas | v1.0.0 publicada |

---

## 7. Especificação funcional do Core (tudo isto está APROVADO)

### 7.1 Motor de descrição (frases detalhadas)

**Padrão definido pelo dono (PT‑BR):**
> **Usuário clicou no botão do mouse [esquerdo] em "Salvar" [botão] na janela chamada "Sem título – Bloco de notas"**

Se a ação foi no touchpad, a palavra muda para **touchpad**:
> **Usuário tocou com [um dedo] no touchpad em "Salvar" [botão] na janela chamada "Sem título – Bloco de notas"**

**EN:**
> **User clicked the [left] mouse button on "Save" [button] in the window named "Untitled – Notepad"**

Regras:
- Sempre que possível, informe: **ator, ação, dispositivo, botão ou dedos, alvo (nome), tipo do alvo, janela (título) e aplicativo (processo)**. Para scroll: **direção e quantidade**. Para teclado: **teclas e texto**.
- **Templates editáveis** em Configurações, com variáveis: `{ator} {acao} {dispositivo} {botao} {dedos} {alvo} {tipo_alvo} {janela} {app} {direcao} {quantidade} {teclas} {texto} {hora}`. O `{ator}` vem "Usuário" por padrão e pode virar o nome da pessoa.
- Cada passo é **editável manualmente** no editor (o texto gerado é só o ponto de partida).
- Se faltar informação, a frase degrada com elegância (ex.: `em elemento sem nome (posição 812, 430)`).
- i18n com arquivos de tradução (ex.: Fluent ou JSON ICU), incluindo plural e gênero.

**Exemplos que o motor deve produzir (PT):**
| Evento | Frase |
|---|---|
| Clique direito | Usuário clicou no botão do mouse [direito] em "relatorio.pdf" [item de lista] na janela chamada "Downloads" |
| Clique duplo | Usuário clicou duas vezes no botão do mouse [esquerdo] em "Fotos" [pasta] na janela chamada "Explorador de Arquivos" |
| Clique do meio | Usuário clicou no botão do mouse [do meio] em "Notícias" [link] na janela chamada "Firefox" |
| Scroll | Usuário rolou a roda do mouse [para baixo] 5 vezes em "Documento" [painel] na janela chamada "Writer" |
| Botões laterais | Usuário clicou no botão do mouse [lateral – voltar] na janela chamada "Chrome" |
| Touchpad, 2 dedos | Usuário tocou com [dois dedos] no touchpad (clique direito) em "Área de trabalho" [painel] |
| Scroll no touchpad | Usuário rolou com [dois dedos] no touchpad [para baixo] em "Página" [documento] na janela chamada "Edge" |
| Gesto | Usuário deslizou [três dedos para cima] no touchpad — abriu a Visão de Tarefas |
| Pinça | Usuário fez pinça para [ampliar] com [dois dedos] no touchpad na janela chamada "Fotos" |
| Teclado | Usuário digitou "relatório 2026" em "Nome do arquivo" [caixa de texto] na janela chamada "Salvar como" |
| Atalho | Usuário pressionou [Ctrl + S] na janela chamada "Sem título – Bloco de notas" |
| Arrastar | Usuário arrastou "foto.png" [item] de "Downloads" até "Área de trabalho" com o botão do mouse [esquerdo] |

> Com **rolagem natural** ligada, a direção dos dedos e a do conteúdo são opostas. Descreva a **direção do conteúdo** e mostre a dos dedos no ícone.

### 7.2 Catálogo de eventos de MOUSE (cada um tem ícone próprio)
Ícone base: **um mouse pequeno em SVG, com o botão usado em destaque colorido**. No scroll, a **roda mostra uma seta animada ↑ ou ↓** (ou ← → na rolagem horizontal).

| ID | Evento | Ícone / animação |
|---|---|---|
| MS‑01 | Clique esquerdo | Botão esquerdo pisca uma vez |
| MS‑02 | Clique direito | Botão direito pisca uma vez |
| MS‑03 | Clique do meio (roda pressionada) | Roda pisca |
| MS‑04 | Clique duplo (respeitar `GetDoubleClickTime` / config do desktop) | Botão pisca 2× |
| MS‑05 | Clique triplo | Pisca 3× |
| MS‑06 | Rolagem para cima / para baixo (contar os "detents"; suportar roda de alta resolução) | Roda com seta ↑ / ↓ animada + contador |
| MS‑07 | Rolagem horizontal (roda inclinável ou Shift+roda) | Setas ← → |
| MS‑08 | Botões laterais X1 / X2 (voltar / avançar) | Botão lateral destacado |
| MS‑09 | Arrastar e soltar (com qualquer botão; limiar `SM_CXDRAG`) | Botão pressionado + seta de trajeto |
| MS‑10 | Pressionar e segurar (long press) | Botão com anel de progresso |
| MS‑11 | Ctrl/Shift/Alt + clique ou roda | Mouse + teclinha do modificador |

### 7.3 Catálogo de GESTOS DE TOUCHPAD (pesquisado: Windows Precision Touchpad, libinput, GNOME, KDE, Cinnamon)
Cada gesto tem **ícone customizado + mini animação** (SVG animado com CSS; ~1 a 1,5 s em loop; 24, 32 e 48 px). O desenho é um retângulo de touchpad com as pontas dos dedos (círculos), mostrando o movimento: rastro, setas e afastamento/aproximação. Respeite `prefers-reduced-motion`. Nos formatos estáticos (PDF/DOCX), use um quadro‑chave com setas.

**Toques e cliques**
| ID | Gesto | Resultado padrão | Notas de detecção |
|---|---|---|---|
| TP‑01 | Toque com 1 dedo | Clique esquerdo | Win: configurável ("Tocar com um dedo para clicar"). Linux: libinput mapeia 1/2/3 dedos → esquerdo/direito/meio; "toque para clicar" vem **desligado por padrão** na maioria dos dispositivos, então leia a config |
| TP‑02 | Toque duplo com 1 dedo | Clique duplo | |
| TP‑03 | Toque com 2 dedos | Clique direito | Win: "Tocar com dois dedos para clicar com o botão direito" |
| TP‑04 | Toque com 3 dedos | **Win: Pesquisa** (configurável: Central de notificações, Reproduzir/pausar, **Botão do meio**, Voltar, Avançar, atalho personalizado). **Linux: clique do meio** (o mapa pode ser trocado para esquerdo/meio/direito) | Descrever o resultado real quando detectável |
| TP‑05 | Toque com 4 dedos | **Win 11: Central de notificações** (Win 10: Central de Ações; configurável). **Linux: o libinput não suporta** | Registrar mesmo assim, se os contatos HID mostrarem 4 dedos |
| TP‑06 | Pressionar o clickpad (clique físico) | Clique esquerdo | Diferenciar de toque: o botão do clickpad aparece no relatório HID / `BTN_LEFT` |
| TP‑07 | Pressionar o canto inferior direito | Clique direito | Win: "Pressionar o canto inferior direito…". Linux: *software buttons* (método "areas") |
| TP‑08 | Clique físico com 2 dedos / 3 dedos | Direito / meio | Linux: método *clickfinger* |
| TP‑09 | Tocar duas vezes e arrastar | Selecionar vários / arrastar | Win: "Tocar duas vezes e arrastar para selecionar vários itens". Linux: *tap‑and‑drag*, com variantes **drag lock** (sticky / timeout) |
| TP‑10 | Pressionar e arrastar (um dedo segura, outro move) | Arrastar | |
| TP‑11 | Segurar (hold) com 1 a 4 dedos | Parar a rolagem cinética / menu de contexto | Linux: `GESTURE_HOLD` (begin/end/cancel) |

**Rolagem e zoom**
| ID | Gesto | Resultado | Notas |
|---|---|---|---|
| TP‑12 | 2 dedos para cima / baixo | Rolagem vertical | Respeitar rolagem natural |
| TP‑13 | 2 dedos para esquerda / direita | Rolagem horizontal | |
| TP‑14 | Rolagem cinética (soltar os dedos em movimento) | Rolagem continua | Agrupar num único passo |
| TP‑15 | Rolagem pela borda (1 dedo na borda) | Rolagem | Linux (*edge scrolling*) e drivers antigos do Win7 |
| TP‑16 | Pinça com 2 dedos (afastar / aproximar) | Ampliar / reduzir | No Windows costuma chegar aos apps como Ctrl+roda; detectar pelos contatos |
| TP‑17 | Rotação com 2 dedos | Girar (apps compatíveis) | Linux: ângulo do `GESTURE_PINCH` |

**Deslizes de 3 e 4 dedos (navegação do sistema)**
| ID | Gesto | Windows 10/11 (padrão) | GNOME 48/50 | KDE Plasma 6 | Cinnamon (Mint) |
|---|---|---|---|---|---|
| TP‑18 | 3 dedos ↑ | Visão de Tarefas | Visão geral de Atividades (2º deslize: grade de apps) | — (sem padrão) | Configurável (ex.: Expo) |
| TP‑19 | 3 dedos ↓ | Mostrar área de trabalho | Sai da visão geral | — | Configurável |
| TP‑20 | 3 dedos ← / → | Alternar apps / janelas | Trocar área de trabalho | Trocar desktop virtual | Configurável (ex.: área de trabalho) |
| TP‑21 | 4 dedos ← / → | Trocar desktops virtuais | — | Trocar desktop virtual | Configurável |
| TP‑22 | 4 dedos ↑ | Visão de Tarefas (padrão do preset) | — | **Visão geral (Overview)** | Configurável |
| TP‑23 | 4 dedos ↓ | Mostrar área de trabalho | — | **Grade de desktops** | Configurável |
| TP‑24 | Pinça com 3 / 4 dedos | — | — (extensões) | — (versões antigas: Overview) | Configurável |

Observações para o catálogo:
- No Windows, os deslizes e toques de 3 e 4 dedos são **configuráveis** (Configurações → Bluetooth e dispositivos → Touchpad → Gestos avançados). Leia a configuração quando possível (registro: pesquise a chave `PrecisionTouchPad`) para descrever a **ação real**.
- Sempre que der para confirmar o efeito (ex.: a janela de Visão de Tarefas apareceu, a área de trabalho mudou), acrescente "— abriu a Visão de Tarefas".
- **Touchscreen** (fora do escopo inicial; deixar a arquitetura pronta): toque, pressionar e segurar, deslizar da borda esquerda/direita, pinça.
- Gestos de 5 dedos: só registrar como "gesto com 5 dedos" se o hardware informar.

### 7.4 Teclado
- Agrupar a digitação contínua num passo: "Usuário digitou '…' em …".
- Atalhos (Ctrl/Alt/Shift/Win/Super + tecla) viram passo próprio, com ícone de teclas.
- **Campos de senha NUNCA são gravados**: `IsPassword` na UIA; papel `password text` na AT‑SPI. Mostre "Usuário digitou a senha (oculta)".
- **Wayland/evdev:** converter os códigos de tecla com o **layout real** do usuário (`xkbcommon` + config do desktop), inclusive ABNT2 (ç, acentos com teclas mortas).
- A gravação de texto digitado pode ser desligada (privacidade).

### 7.5 Captura, destaque e trilha do mouse
- Print do monitor onde ocorreu a ação (multi‑monitor, **DPI correto**; Per‑Monitor DPI Aware v2 no Win10+), ou só da janela ativa (opção).
- Destaque do clique: círculo/anel + número do passo + mini ícone do dispositivo.
- **Trilha do mouse** entre um passo e outro, com estilos: **pontilhada**, **sólida** ou **semitransparente**. Cor, espessura e opacidade configuráveis, e liga/desliga por passo. Suavização da trilha (ex.: Catmull‑Rom ou Douglas‑Peucker para simplificar).
- **Zoom inteligente:** recorte automático em volta do clique (opcional, por passo).
- Print **antes** do clique (padrão), **depois**, ou **ambos**.
- Sem limite prático de passos (o PSR limitava a 25 por padrão).

### 7.6 Editor
- A timeline de passos fica à esquerda e o passo selecionado, grande, à direita.
- Funções:
  - reordenar (arrastar), apagar, duplicar e mesclar passos;
  - editar o texto, inserir um passo manual (com print novo ou imagem importada);
  - recortar a imagem e anotar (seta, retângulo, texto, numeração, marca‑texto);
  - desfoque manual;
  - desfazer/refazer ilimitado;
  - salvamento automático.
- Pausar/retomar a gravação; comentário de texto em qualquer passo (comentário de voz é opcional/futuro).
- Busca nos passos.

### 7.7 Privacidade (desfoque automático)
- Desfoque automático de: campos de senha; **e‑mail, CPF, CNPJ, cartão de crédito (validação Luhn), telefone, CEP, IP**, por **regex** aplicado no texto da UIA/AT‑SPI e no OCR local, com as regiões mapeadas na imagem.
- Lista de **apps/janelas ignorados** (ex.: gerenciador de senhas, banco).
- Tudo **offline**. Zero telemetria por padrão (relatório de falhas só opcional, desligado).
- Indicador de "gravando" sempre visível.

### 7.8 Exportar
| Formato | Detalhes |
|---|---|
| **HTML (padrão)** | Arquivo único autocontido (imagens em base64/WebP, CSS e SVG inline, ícones animados), tema claro/escuro, índice, navegação por teclado, abre offline em qualquer navegador |
| **PDF** | Diagramação bonita (sugestão: Typst embutido), capa, índice, numeração |
| **DOCX** | Editável no Word/LibreOffice, com estilos de título |
| **Markdown** | Pasta com `.md` + imagens, e opção "copiar como Markdown" (Jira, GitHub, Notion, Teams) |
| **GIF / MP4** | Modo "replay": slideshow com zoom no clique, trilha e ícones animados |
| **MHT** | Compatível com o PSR |
| **ZIP** | Pacote com HTML + imagens + `.mypath` |
| **.mypath** | Formato próprio editável: ZIP com `manifest.json` versionado (schema), imagens e eventos brutos |

Todas as exportações incluem, opcionalmente, um rodapé com a **licença escolhida** (seção 9) e informações do sistema.

### 7.9 Importar
- **`.mht` / `.zip` do Gravador de Passos da Microsoft:** extrair as imagens e as descrições (o MHT do PSR tem as imagens embutidas e um bloco XML com as ações; analise arquivos reais de exemplo e crie testes com eles).
- `.mypath` (inclusive versões antigas, com migração de schema).
- `.zip` exportado pelo próprio My Path.
- **Pasta de imagens** ou várias imagens → um passo por imagem, para montar o guia manualmente.
- Arrastar e soltar arquivos na janela.

### 7.10 Setup de um clique, distribuição e atualização
- **Windows:**
  - instalador **NSIS** que instala sem perguntas desnecessárias e **abre o app ao terminar**;
  - versão **portátil (.exe único)**;
  - **winget** (`winget install MyPath.MyPath`);
  - **Legacy Win7** com o WebView2 Fixed 109 embutido.
- **Linux:** `.deb`, `.rpm` e AppImage. Depois de instalar, a tela "Preparar meu sistema" resolve as permissões com um clique (seção 5, desafio 7).
- **Atualização automática** com `tauri-plugin-updater` (assinada), com opção de desligar.
- Releases separadas por edição: `MyPath-Core`, `MyPath-Support`, `MyPath-QA`, `MyPath-Docs`, `MyPath-Education`, cada uma para Windows x64, Windows 7 legacy, deb, rpm e AppImage.

### 7.11 UI/UX
- **Barra flutuante** discreta: ⏺ Gravar / ⏸ Pausar / ⏹ Parar / contador de passos ao vivo / flash sutil a cada captura.
- **Atalho global** configurável (padrão `Ctrl+Shift+R`).
- Ao parar, abre o editor automaticamente.
- Design moderno, **tema claro/escuro** seguindo o sistema, animações suaves (60 fps), acessível (contraste, navegação por teclado, leitor de tela).
- Inicia em menos de 1 s (meta).
- Modo replay dentro do app.
- **Informações do sistema** anexáveis (SO, versão, resolução, app em foco).

---

## 8. Edições (mesmo repositório, recursos somados ao Core)

### 🛠️ My Path Support
1. Informações completas do sistema (hardware, SO, drivers, apps instalados).
2. Coleta de logs.
3. Enviar por e‑mail / abrir ticket (mailto + pacote zip; integrações genéricas por URL/API configurável).
4. **Kit de captura para o cliente final:** o técnico gera um executável/link pré‑configurado; o cliente clica uma vez, grava e o arquivo volta para o técnico.
5. **Erros do sistema ligados a cada passo:** Visualizador de Eventos (Windows) / `journalctl` (Linux), filtrados pelo horário da gravação.
6. **Diagnóstico de rede automático:** IP, DNS, gateway, proxy, ping e teste de portas.

### 🐞 My Path QA
1. Formato de bug report (passos para reproduzir, esperado × obtido, ambiente).
2. Exportar para Jira / GitHub Issues (Markdown + API opcional com token do usuário).
3. **Exportar como script de automação:** pywinauto, AutoHotkey, xdotool/ydotool e Playwright (para web), com base nos seletores da UIA/AT‑SPI.
4. **Diferença visual entre gravações:** comparação de prints (diff por pixel/SSIM) entre versões do software.
5. **Execução de casos de teste:** importar um roteiro; marcar cada passo como ✅ passou / ❌ falhou, com gravidade.

### 📚 My Path Docs
1. Capítulos e seções.
2. Exportação premium com marca/logo da empresa e temas.
3. **Regravar um passo só:** atualiza o passo 7 e a numeração/texto se ajustam.
4. **Publicar como site estático com busca** (offline ou GitHub Pages).
5. **Histórico de versões do documento**, com comparação entre v1 e v2.

### 🎓 My Path Education
Modos:
- **Aula (professor):** grava a tarefa e ela vira uma atividade com objetivo, dificuldade e tempo estimado.
- **Prática guiada (aluno):** uma sobreposição destaca **onde clicar no próximo passo**; o app confere pelo elemento da UIA/AT‑SPI se o aluno clicou certo, avança sozinho e mostra dica no erro. (Atenção: sobreposições "click‑through" sempre no topo são limitadas no Wayland/GNOME. Pesquise e documente o plano B, ex.: painel lateral em vez de sobreposição.)
- **Quiz "Onde você clicaria?":** o aluno clica no print; a resposta é conferida pela área do elemento.
- **Avaliação:** o aluno grava fazendo a tarefa sozinho; um **algoritmo** compara com o gabarito (elementos, ordem, janelas; tolerância configurável) e gera nota e relatório.

Recursos:
- Exportação **SCORM 1.2/2004 e H5P** (Moodle, Google Classroom, Canvas).
- **Acessibilidade:** fonte grande, alto contraste, fonte para dislexia (ex.: OpenDyslexic, licença OFL), **leitura em voz pelo sintetizador do sistema** (SAPI no Windows, Speech Dispatcher no Linux), que não é IA generativa.
- **Modo laboratório:** instalação em massa (MSI/linha de comando silenciosa, .deb), sem conta, 100% offline.
- **Painel do professor:** importa as gravações da turma e mostra um mapa de calor de onde a turma mais erra.
- **Trilhas e certificados:** atividades em sequência e certificado em PDF.
- **Modo seguro para menores (LGPD):** desfoque reforçado, nenhum dado sai do PC, aluno identificado só por código.

---

## 9. Licenciamento (IMPORTANTE: leia a explicação)

O dono pediu "Apache‑2.0 para uso comercial e MIT para uso não comercial, com opção de escolher no início da gravação". A pesquisa mostrou o seguinte:
- **MIT e Apache‑2.0 permitem uso comercial.** Nenhuma licença open source pode proibir o uso comercial (item 6 da Open Source Definition: "nenhuma discriminação contra áreas de atuação").
- O que o dono descreveu corresponde ao **licenciamento duplo "MIT OR Apache‑2.0"**: quem usa escolhe. É o padrão do próprio Rust e do Tauri. A Apache‑2.0 acrescenta **concessão e retaliação de patentes**; as duas trazem **isenção de garantia**, que é o que protege o autor.

**Implemente assim:**
1. Código do repositório: `LICENSE-MIT` + `LICENSE-APACHE` + `NOTICE`. No README: "licenciado sob **MIT OR Apache‑2.0**, à sua escolha". Em cada `Cargo.toml` / `package.json`: `"MIT OR Apache-2.0"`.
2. Contribuições: cláusula "inbound = outbound" no CONTRIBUTING + **DCO** (`Signed-off-by`).
3. **No app, no início da gravação** (com "lembrar minha escolha"): seletor **"Licença deste documento"**, porque o guia gravado é conteúdo do usuário e não código. Opções:
   - MIT;
   - Apache‑2.0;
   - **CC BY 4.0**;
   - **CC BY‑NC 4.0** (esta permite **proibir uso comercial do documento**, que era a intenção original);
   - CC0;
   - "Todos os direitos reservados".

   A licença escolhida vai para os metadados e o rodapé de todas as exportações.
4. Não é aconselhamento jurídico. Sugira que o dono consulte um advogado se for comercializar.

---

## 10. Qualidade, segurança e CI

- **CI (GitHub Actions):** `cargo fmt`, `clippy -D warnings`, `cargo test`, `cargo-deny` (licenças e vulnerabilidades), `svelte-check`, `eslint`, testes de UI (Vitest + Playwright no frontend), builds de Windows (runner windows‑latest), Linux (ubuntu runner + container Debian 13 para testes) e legacy‑win7.
- **Testes do motor de gestos com "gravações sintéticas"**: arquivos de eventos brutos (contatos HID / evdev) que viram testes de regressão. Peça que os relatórios do Probe incluam esses eventos brutos.
- **Testes do motor de frases:** snapshot por idioma.
- **Testes de importação:** arquivos reais do PSR.
- **Desempenho:** benchmark de latência do print e de CPU/RAM no CI quando possível.
- **Segurança:** permissões mínimas do Tauri (capabilities), CSP rígida, atualizações assinadas, nenhum dado enviado sem ação explícita.
- **Versionamento:** SemVer + CHANGELOG (Keep a Changelog) + Conventional Commits.

---

## 11. Ambiente de desenvolvimento da IA (limites que VOCÊ tem)

- O ambiente onde você roda (Arena: **Debian 13 trixie**, sem tela e sem touchpad) **não testa** hooks, gestos nem captura de tela reais.
- Use o ambiente para: compilar a parte Linux, rodar testes unitários, testes com eventos sintéticos e, se útil, Xvfb/Weston headless para testes básicos de X11/Wayland.
- **Os testes reais são feitos por mim**, no meu hardware, com o Probe e os roteiros. Escreva roteiros curtos e à prova de leigo.
- Para o Windows, confie no CI do GitHub (runners Windows) e nos meus testes.

---

## 12. Arquivos de memória do projeto (mantenha SEMPRE atualizados)

| Arquivo | Conteúdo |
|---|---|
| `docs/PROGRESSO.md` | Fase atual, tarefas feitas/pendentes, último incremento, próximo passo |
| `docs/DECISOES.md` | Registro de decisões (data, decisão, motivo, alternativas) |
| `docs/LIMITACOES.md` | Limites técnicos conhecidos e planos B |
| `docs/COMPATIBILIDADE.md` | Matriz SO × sessão × desktop × recurso |
| `docs/GLOSSARIO.md` | Termos técnicos explicados em português simples, para mim |
| `docs/COMO_TESTAR.md` | Roteiros de teste para leigo |

**No início de cada sessão:** leia esses arquivos e me diga em 3 linhas onde paramos.

---

## 13. SUA PRIMEIRA TAREFA (agora)

1. Confirme em 5 linhas que entendeu o projeto, as regras para leigo e que a **Fase 1** vem primeiro.
2. Liste as **3 perguntas mais importantes**, se houver alguma dúvida real (senão, siga).
3. **Me guie para criar a conta e o repositório no GitHub** (`my-path`, público), com cliques exatos. Se a ferramenta tiver integração com o GitHub, use‑a.
4. Crie a estrutura do monorepo (seção 4), os arquivos de licença (seção 9), os arquivos de memória (seção 12) e um CI mínimo que compila um "Probe" vazio para Windows e Linux.
5. Comece o **Desafio 1 + 2** (captura de entrada + mouse × touchpad) no Probe e me entregue o primeiro **roteiro de teste**.

---

## Anexo A: Referências pesquisadas (confira antes de usar)

- Tauri 2.12 removeu o Win7: https://v2.tauri.app/blog/tauri-2.12/
- WebView2 e versões no Tauri: https://v2.tauri.app/reference/webview-versions/
- WebView2 109 = último no Win7: https://blogs.windows.com/msedgedev/2022/12/09/microsoft-edge-and-webview2-ending-support-for-windows-7-and-windows-8-8-1/
- Alvos Rust win7: https://doc.rust-lang.org/rustc/platform-support/win7-windows-msvc.html
- Gestos do Windows (touch e touchpad): https://support.microsoft.com/en-us/windows/touch-gestures-for-windows-a9d28305-4818-a5df-4e2b-e5590f850741
- Coleções HID do Precision Touchpad: https://learn.microsoft.com/en-us/windows-hardware/design/component-guidelines/windows-precision-touchpad-required-hid-top-level-collections
- `IUIAutomation::ElementFromPoint`: https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomation-elementfrompoint
- `GraphicsCaptureSession` (Win10 1803+): https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.graphicscapturesession
- Gestos do libinput (pinch / swipe / hold): https://wayland.freedesktop.org/libinput/doc/latest/gestures.html
- Toque para clicar, tap‑and‑drag e drag lock: https://wayland.freedesktop.org/libinput/doc/latest/tapping.html
- Portal ScreenCast (`cursor_mode` Metadata): https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.impl.portal.ScreenCast.html
- AT‑SPI no Wayland com coordenadas relativas à janela: https://bugzilla.redhat.com/show_bug.cgi?id=1517301
- GNOME 50 só Wayland (Ubuntu 26.04 / Fedora 44): https://www.theregister.com/software/2026/03/19/gnome-50-debuts-with-x11-axed-wayland-front-and-center/5224877
- Gestos do GNOME: https://help.gnome.org/users/gnome-help/stable/touchscreen-gestures.html.en
- Cinnamon no Wayland (gestos não suportados): https://forums.linuxmint.com/viewtopic.php?t=411761
- Crate `atspi` (Rust puro): https://crates.io/crates/atspi
- Crate `xcap`: https://lib.rs/crates/xcap
- Open Source Definition, item 6: https://opensource.org/osd
