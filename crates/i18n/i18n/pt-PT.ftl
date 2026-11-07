# WoodSketch em português de Portugal. Cada chave é a que o código pede; uma chave que falte numa
# tradução aparece em inglês em vez de não aparecer nada.
#
# Uma chave é `area-coisa`, e uma dica é a mesma chave com `-hint`. Mantenha a chave quando o
# texto muda e as traduções acompanham-se sozinhas; mude uma chave só quando a coisa que ela
# nomeia passou a ser outra, para que uma tradução desatualizada seja notada em vez de ficar.
#
# A pontuação pertence à tradução. Um `...` no fim significa "isto abre algo que vai pedir mais",
# uma convenção do Windows que um tradutor pode livremente deixar cair.

## The menu bar

menu-file = Ficheiro
menu-edit = Editar
menu-view = Ver
menu-camera = Câmara
menu-draw = Desenhar
menu-tools = Ferramentas
menu-window = Janela
menu-help = Ajuda

## File

file-start-page = Página Inicial
file-start-page-hint = Novo, Abrir, Recentes e os exemplos; Esc volta para cá
file-new = Novo
file-open = Abrir...
file-open-recent = Abrir Recente
file-save = Guardar
file-save-as = Guardar Como...
file-import-skp = Importar Modelo SketchUp...
file-import-skp-hint = Acrescenta um .skp (SketchUp 2021 ou posterior) a este desenho
file-print-build-sheets = Imprimir Folhas de Montagem...
file-print-build-sheets-hint = Uma página por peça com os seus passos, depois como tudo se junta, com desenhos explodidos; abre no navegador para imprimir
file-export = Exportar
file-export-obj = OBJ...
file-export-obj-hint = O modelo 3D com as cores dos materiais, para renderizadores e outros programas de modelação
file-export-stl = STL...
file-export-stl-hint = O modelo 3D, para impressão 3D
file-export-dxf = DXF para CNC...
file-export-dxf-hint = Cada peça ao alto, vista de frente: o contorno, os furos passantes e as cavidades em camadas por profundidade
file-export-svg = SVG para CNC...
file-export-svg-hint = Cada peça ao alto, vista de frente: o contorno a preto, os furos a branco, as cavidades a cinzento
file-export-units = Unidades
file-export-inches = Polegadas
file-export-millimeters = Milímetros
file-exit = Sair

## Edit

edit-undo = Desfazer
edit-redo = Refazer
edit-cut = Cortar
edit-copy = Copiar
edit-paste = Colar
edit-paste-hint = Segue o rato; clique para o largar
edit-paste-in-place = Colar no Local
edit-duplicate = Duplicar
edit-duplicate-hint = Uma cópia logo ao lado
edit-delete = Eliminar
edit-delete-guides = Eliminar Guias
edit-select-all = Selecionar Tudo
edit-select-none = Não Selecionar Nada
edit-make-group = Criar Grupo
edit-make-component = Criar Componente...
edit-explode = Explodir

## View

view-grid = Grelha
view-wood-movement = Movimento da Madeira
view-wood-movement-hint = Quanto cada peça de madeira maciça move contra o veio: vermelho onde uma peça que a encontra a prende, verde onde está livre
view-theme = Tema
view-theme-light = Claro
view-theme-dark = Escuro
view-theme-system = Seguir o Sistema
view-theme-hint = O Escuro cansa menos os olhos à noite; Seguir o Sistema acompanha o computador
view-units = Unidades
view-units-hint = Como os comprimentos aparecem, e o que significa um número escrito sem unidade; guardado com o desenho
view-guides = Guias
view-guides-hint = Mostrar ou ocultar todos os guias (não se liga aos que estão ocultos)
view-dimensions = Cotas
view-dimensions-hint = Mostrar ou ocultar todas as cotas
view-text = Texto
view-text-hint = Mostrar ou ocultar todas as notas de texto
view-default-tray = Bandeja Predefinida
view-language = Idioma
view-language-hint = O idioma em que o próprio WoodSketch está escrito; o seu desenho fica intacto

## Camera

camera-standard-views = Vistas Predefinidas
camera-view-iso = Iso
camera-view-top = Topo
camera-view-front = Frente
camera-view-right = Direita
# The toolbar's view buttons, whose tooltip names the view: "Top view".
camera-view-hint = Vista: { $view }
camera-zoom-extents = Zoom a Tudo

## Tools

tools-cut-list = Lista de Corte...

## Window

window-default-tray = Bandeja Predefinida
window-cut-list = Lista de Corte

## Help

help-check-for-updates = Verificar Atualizações...
help-check-at-startup = Verificar atualizações ao iniciar
help-check-at-startup-hint = Uma vista rápida aos lançamentos do WoodSketch no GitHub quando abre; as novas versões descarregam-se em segundo plano
help-whats-new = Novidades...
help-whats-new-hint = O que as últimas atualizações trouxeram, da mais recente para a mais antiga
help-release-notes = Notas de Versão
help-release-notes-hint = As notas de todas as versões, no GitHub
help-report-problem = Comunicar um Problema...
help-report-problem-hint = Diga-nos o que correu mal: abre uma issue no GitHub com detalhes que ajudam, nada de privado
help-send-feedback = Enviar Comentários...
help-send-feedback-hint = Uma ideia ou qualquer outra coisa: abre uma issue no GitHub
help-problems-and-ideas = Problemas e Ideias
help-problems-and-ideas-hint = Tudo o que foi reportado ou sugerido, e o que foi corrigido, no GitHub
help-discussions = Discussões
help-discussions-hint = Pergunte como fazer algo, diga o que o WoodSketch deverá tratar a seguir, ou mostre uma peça que fez
help-about = Acerca do WoodSketch

## The About window

about-title = Acerca do WoodSketch
about-version = WoodSketch { $version }
about-whats-new = Novidades
about-what-it-is = Um modelador 3D leve para marcenaria, no espírito do SketchUp.
about-not-affiliated = Sem afiliação nem endosso da Trimble.

## The status bar

status-reloaded = Recarregado: o ficheiro mudou fora do WoodSketch
status-reloaded-hint = Ctrl+Z volta ao desenho tal como estava antes do recarregamento
status-dismiss = Dispensar
status-whats-new = Novidades
status-whats-new-hint = O que veio nesta atualização, e em todas as que saltou
status-updated = Atualizado para { $version }
status-downloading = A descarregar o WoodSketch { $version }...
status-restart-to-update = Reiniciar para atualizar para { $version }
status-restart-to-update-hint = Instala a nova versão e reabre o WoodSketch (ser-lhe-á pedido que guarde primeiro)

## The tools' own names, shown in the Tools menu and in each toolbar button's tooltip.
##
## These are SketchUp's names, because most people arriving here already know them from there.
## SketchUp has its own translation, so if it names a tool in your language, that name is the one
## your readers will be looking for -- use it rather than a fresh translation of the English.
## Leaving one in English is a reasonable choice too, if that is what people in your language
## actually say.

tool-name-select = Selecionar
tool-name-eraser = Borracha
tool-name-paint-bucket = Balde de Tinta
tool-name-line = Linha
tool-name-arc = Arco
tool-name-rectangle = Retângulo
tool-name-circle = Círculo
tool-name-polygon = Polígono
tool-name-center-arc = Arco Central
tool-name-three-point-arc = Arco de 3 Pontos
tool-name-pie = Setor
tool-name-rotated-rectangle = Retângulo Rotacionado
tool-name-board = Tábua
tool-name-push-pull = Empurrar/Puxar
tool-name-move = Mover
tool-name-rotate = Rodar
tool-name-scale = Escala
tool-name-offset = Compensar
tool-name-tape-measure = Fita Métrica
tool-name-protractor = Transferidor
tool-name-dimension = Cota
tool-name-text = Texto
tool-name-orbit = Órbita
tool-name-pan = Panorâmica
tool-name-zoom = Zoom
tool-name-walk = Percorrer

# A toolbar button's tooltip: the tool's name and the key that picks it. $key is the same on
# every keyboard, so keep it as it is.
tool-hint = { $tool } ({ $key })
# The same, for a tool that isn't built yet.
tool-hint-coming-soon = { $tool } ({ $key }): brevemente
# Said at the bottom of the window when someone picks a tool that isn't built yet.
tool-coming-soon = { $tool } ({ $key }) chega brevemente.
# Said when a length or an angle is typed while holding a tool that has nothing to do with one.
tool-takes-no-typed-values = { $tool } não aceita valores escritos.

## The toolbar. Each of these is a tooltip, and carries the keyboard shortcut in brackets; the
## shortcut itself is the same on every keyboard, so leave it as it is and translate around it.

toolbar-new-hint = Novo (Ctrl+N)
toolbar-open-hint = Abrir (Ctrl+O)
toolbar-save-hint = Guardar (Ctrl+S)
toolbar-undo-hint = Desfazer (Ctrl+Z)
toolbar-redo-hint = Refazer (Ctrl+Y)
toolbar-zoom-extents-hint = Zoom a Tudo (Shift+Z)
toolbar-cut-list-hint = Lista de Corte: peças e diagramas de corte

## The Default Tray, down the right-hand side, and the name of each panel in it

tray-title = Bandeja Predefinida
tray-hide-hint = Esconder a bandeja (Janela > Bandeja Predefinida)
tray-board = Tábua
tray-entity-info = Informação da Entidade
tray-joinery = Junções
tray-edge-profile = Perfil de aresta
tray-templates = Modelos
tray-materials = Materiais
tray-tags = Etiquetas
tray-outliner = Estrutura
# The sizes of whichever template is selected: "Cabinet sizes", "Drawer sizes". The template's own
# name is not translated yet, so it arrives in English.
tray-template-sizes = Tamanhos de { $template }

## The What's New window (Help > What's New), shown once after an update

whats-new-title = Novidades no WoodSketch
# Marked beside the version that is running.
whats-new-this-one = (esta)
# How many versions have arrived since the one that ran last. Two messages rather than one with a
# plural, because the singular drops the number altogether: "One update since 0.1.64:".
whats-new-one-since = Uma atualização desde { $from }:
whats-new-many-since = { $count } atualizações desde { $from }:
whats-new-recent = As atualizações mais recentes:
# "Done" rather than "Close", because clicking it marks the notes as read.
whats-new-done = Feito
whats-new-done-hint = Marca-as como lidas
whats-new-every-version = Todas as versões no GitHub

## The Outliner panel: where you are in the drawing, and what is at this level

outliner-empty = Ainda não há grupos nem componentes.
outliner-group = Grupo
outliner-group-hint = Criar Grupo (Ctrl+G)
outliner-component = Componente
outliner-component-hint = Criar Componente (G)
outliner-explode-hint = Transformar os grupos selecionados de novo em geometria solta
outliner-auto-group = Agrupar automaticamente as novas tábuas
outliner-auto-group-hint = Quando Empurrar/Puxar transforma uma face isolada num sólido, criar um grupo
# The start of the trail showing which group is open: "Model > Case > Leg".
outliner-model = Modelo
outliner-close-group = Fechar grupo (Esc)
# Marks a row whose tag is hidden, after the name.
outliner-hidden = (oculta)
outliner-open-hint = Aberto: está a trabalhar lá dentro
outliner-needs-material-hint = Precisa de um material para a lista de corte

## The Board panel: the stock a new board is drawn from

board-material = Material
board-pick-material = Escolha um material
# A material with what it comes as, when its name doesn't already say: "Pine (boards)".
board-with-noun = { $name } ({ $noun })
board-noun-lumber = madeira estrutural
board-noun-boards = tábuas
board-noun-sheets = painéis
# The headings the material dropdown groups by.
board-kind-lumber = Madeira estrutural: 2x4, 2x6...
board-kind-boards = Tábuas: 3/4", 1"... de espessura
board-kind-sheets = Painéis: contraplacado, MDF
board-from-library-hint = Da biblioteca: acrescenta-o ao desenho
board-stock = Stock
board-custom-size-open = Tamanho personalizado...
board-custom-size-open-hint = Acrescentar um tamanho em que este material existe (ou escreva-o na caixa de Medições)
board-lay-it = Deitar
board-flat = Plana
board-on-edge = De canto
board-on-edge-hint = Ctrl alterna durante o desenho
board-lumber-note = A madeira estrutural é designada pelo tamanho nominal; o tamanho real está entre parênteses. Desenhe qualquer comprimento: a lista de corte escolhe a tábua a comprar (8', 10'...). Para um tamanho único, escreva-o, como 3,6 1/2; para manter um tamanho na lista, use Tamanho personalizado.
board-sheet-note = Desenhada como um retângulo com esta espessura. Para 2x4 e outros tamanhos de estrutura, escolha um material de Madeira estrutural, como Pine Lumber.
board-custom-size = Tamanho personalizado
board-thickness = Espessura
board-width = Largura
board-add-to = Acrescentar a { $material }
board-cancel = Cancelar
board-custom-note = É acrescentado aos tamanhos do material, para a lista de corte o conhecer.

## The Tags panel

tags-add-hint = Acrescentar uma etiqueta
tags-add-folder-hint = Acrescentar uma pasta (o clique com o botão direito numa etiqueta serve para a meter numa)
tags-untagged = Sem etiqueta
tags-fold = Recolher
tags-unfold = Expandir
# Two whole sentences rather than one built from pieces: a sentence assembled out of fragments
# only ever comes out right in the language it was assembled for.
tags-hide-folder = Esconder tudo o que está nesta pasta
tags-hide-tag = Esconder tudo o que tem esta etiqueta
tags-show = Voltar a mostrar
tags-rename-hint = Clique duplo para renomear; arraste a linha para uma pasta (ou clique com o botão direito) para a mover
tags-delete-folder-hint = Eliminar pasta (o que está dentro sobe)
tags-delete-tag-hint = Eliminar etiqueta (os seus grupos ficam Sem etiqueta)
tags-move-to = Mover para
tags-top-level = Nível de topo
tags-no-folders = Ainda não há pastas: acrescente uma com o botão da pasta.

## Entity Info: what is selected, its name, tag and material

entity-nothing-selected = Nada selecionado.
entity-name = Nome
entity-text = Texto
entity-tag = Etiqueta
entity-dimension = Cota  { $length }
# Loose geometry — edges, faces, guides — rather than groups.
entity-loose =
    { $count ->
        [one] { $count } entidade solta
       *[other] { $count } entidades soltas
    }
entity-selected = { $count } selecionados ({ $groups } grupos/componentes)
entity-tags-note = As etiquetas vão para grupos e componentes. Agrupe isto primeiro (Ctrl+G).
entity-material = Material
entity-material-default = Predefinido

## The Materials panel, and the editor behind the pencil

materials-add-hint = Acrescentar um material
materials-none = Sem material (predefinido)
materials-none-hint = O aspeto predefinido simples. Pintar com ele remove a tinta. Não é um material editável: acrescente um com +.
# $kind is the sort of material (Solid wood, Sheet good); $pencil is the pencil icon.
materials-paint-hint = { $kind }. Clique para pintar com ele; { $pencil } para o editar.
materials-edit-hint = Editar este material
materials-empty = Ainda não há materiais. Clique { $plus } para acrescentar um, como Red Oak ou Plywood.
materials-none-selected = Nenhum material selecionado: pintar remove a tinta. Escolha acima um material para pintar com ele.
materials-editing = A editar { $name }
materials-close-editor = Fechar o editor
materials-library = Biblioteca
materials-new-blank = Novo material em branco
materials-name = Nome
materials-color = Cor
materials-type = Tipo
materials-grain = Veio
materials-has-grain = Tem veio
materials-has-grain-hint = As peças têm de seguir o veio quando são cortadas
materials-thicknesses = Espessuras
materials-sheet-sizes = Tamanhos de painel
materials-board-lengths = Comprimentos de tábua
materials-board-widths = Larguras de tábua
materials-lengths = Comprimentos
materials-not-cut = Não se corta de stock; a lista de corte conta as peças.
materials-moves-like = Move como
materials-moves-like-hint = Para a verificação do movimento da madeira: a espécie cujo movimento sazonal partilha
materials-movement-custom = Personalizado
materials-movement-unknown = Desconhecido
materials-sections = Secções ({ $shown } de { $total })
materials-sections-note = Tamanhos nominais e reais em que existe; cada peça recebe o menor que serve.
materials-delete = Eliminar material
materials-delete-hint = O que estava pintado volta a ficar sem material

## Which way a face or an edge points in the drawing.
##
## Two forms of each, because English puts them in sentences two ways: "Top-left" starts a name,
## "around the top" sits inside one. A language that doesn't change the word can give both the
## same text; one that changes it more than English does has somewhere to put each form.

side-top = Topo
side-bottom = Fundo
side-left = Esquerda
side-right = Direita
side-front = Frente
side-back = Traseira
side-top-lower = topo
side-bottom-lower = fundo
side-left-lower = esquerda
side-right-lower = direita
side-front-lower = frente
side-back-lower = traseira

## The Edge profile panel

profile-assembly = Um conjunto: faça o perfil das peças que estão dentro.
profile-shape = Forma
profile-size = Tamanho
profile-around-face = À volta de uma face
profile-along-length = Ao longo do comprimento
# A corner named by the two faces meeting at it: "Top-left".
profile-corner = { $first }-{ $second }
# Which face the edges are taken around: "Around the top".
profile-around = À volta da face { $side }
# One edge of that face: "Left edge".
profile-edge = Aresta { $edge }
# $profile is the router bit's name (Roundover, Chamfer), which is not translated yet.
profile-rout = Fresar o { $profile }
profile-routed = { $size } { $profile } fresado.
profile-edge-profiles = Perfis de aresta

## Entity Info's part section: size, stock, grain, reference marks, cut and movement

part-assembly = Um conjunto: as suas peças são os grupos e componentes que estão dentro.
part-size = Tamanho
part-size-hint = Comprimento (ao longo do veio) x largura x espessura
part-stock = Stock
part-needs-material = Precisa de um material
part-stock-line = { $material }, { $stock }
part-stock-glued-up = { $material }, { $stock }, colado a partir de { $boards }
part-grain = Veio
part-grain-along-side = Ao longo do lado { $length }
part-grain-along-length = Ao longo do comprimento
part-grain-along-length-hint = O lado mais comprido (o caso habitual)
part-reference = Referência
part-reference-hint = A face, a aresta e a extremidade de que a folha de montagem mede (as marcas de face e aresta de um marceneiro)
part-reference-face-edge = Face { $face }, aresta { $edge }
part-reference-end = Extremidade { $end }
part-reference-not-set = não definido
# The end is the one mark with no default, so saying it isn't set would be misleading.
part-reference-as-drawn = deixada como desenhada
part-pick = Escolher
part-pick-hint = Depois clique na peça: uma face larga define a face de referência, uma aresta comprida a aresta de referência, uma extremidade a extremidade de referência
part-clear = Limpar
part-cut = Corte
part-cut-unknown-hint = As verificações de movimento assumem corte tangencial, que é o que move mais
part-cut-flat-hint = Corte tangencial: os anéis de crescimento atravessam a extremidade; move o mais
part-cut-quarter-hint = Os anéis de crescimento correm de alto a baixo na extremidade; move cerca de metade
part-movement = Movimento
part-free-to-move = Fixado de modo a poder mover-se
part-free-to-move-hint = Botões de tampo, figure-8s ou parafusos em furos oblongos, não colado nem aparafusado fixo: está livre para mover contra o veio
part-glue-up = Colar
part-board-one = uma tábua
part-boards-n = { $count } tábuas
part-glue-auto = Automático ({ $boards })
part-glue-one = Uma tábua
part-glue-from = A partir de { $count } tábuas
part-edge-joint = Junta de arestas
# $piece is a biscuit or a loose tenon, named by the joinery; English makes it plural with an s.
part-edge-joint-hint = { $piece } ao longo de cada emenda, aproximadamente a cada { $spacing }
part-edge-glued-hint = Colada de aresta a aresta: uma linha de cola bem aplainada é mais forte do que a madeira

## The Templates panel: a cabinet, drawer, face frame or table from a few sizes

template-wood = Madeira
# $template is the template's own name (Cabinet, Drawer), not translated yet.
template-add = Acrescentar o { $template }
template-add-hint = Construído ao lado do desenho, como um grupo; um Desfazer tira-o
template-change-size = Mudar o tamanho
template-change-size-hint = Constrói-o de novo com estes tamanhos e esta madeira, onde está. As alterações feitas à mão lá dentro perdem-se.
template-width = Largura
template-height = Altura
template-depth = Profundidade
template-stock = Stock
template-top = Tampo
template-shelves = Prateleiras
template-adjustable = Reguláveis (em pinos de prateleira)
template-back = Fundo
template-back-ply = Contraplacado de 1/4"
template-joints = Juntas
template-corners = Cantos
template-stiles = Montantes
template-rails = Travessas
template-legs = Pernas
template-aprons = Cinturas
template-middle-rails = Travessas intermédias
template-fronts = Frentes
template-pulls = Puxadores
template-doors = Portas
template-hinge = As portas simples dobram para
template-drawer-corners = Cantos da gaveta
# The openings in a cabinet, named top-down as you look at it. A cabinet with no shelves has
# just the one, so it needs no number.
template-opening = Abertura
template-opening-bottom = Abertura inferior
template-opening-top = Abertura superior
template-opening-n = Abertura { $n }
template-note-cabinet = Lados, tampo, fundo e prateleiras no stock indicado; medidas exteriores. As gavetas correm em corrediças laterais; portas com mais de 24" vêm aos pares.
template-note-drawer = Uma caixa com fundo de contraplacado de 1/4" em ranhuras; medidas exteriores.
template-note-face-frame = Montantes com a altura total, travessas entre eles.
template-note-table = Pernas quadradas, cinturas de 3/4" de espessura, tampo saliente 1"; altura até ao topo do tampo.

## The Joinery panel: the joint between two selected parts
##
## $part and $into are the parts' own names, as the drawing calls them, and $kind and $detail
## are the joinery's own words. They arrive as they are.

joinery-kind-let-in = Rasgo, ranhura ou rebaixe
joinery-kind-overlap = Cortar onde se sobrepõem
joinery-kind-mortise-tenon = Caixa e espiga
joinery-kind-dowels = Cavilhas
joinery-kind-biscuits = Biscuits
joinery-kind-loose-tenons = Espigas soltas (estilo Domino)
joinery-kind-pocket-screws = Parafusos de bolso
joinery-kind-dovetails = Rabos de andorinha
joinery-kind-splines = Chave de encaixe
joinery-kind-box-joint = Junta de dentes
joinery-kind-half-lap = Meia-madeira

# How the two parts have to be sitting before each joint can be made.
joinery-how-let-in = Encoste a extremidade ou a aresta de uma peça à outra (uma prateleira contra um lado, um painel contra um poste). Entra pela profundidade: um rasgo atravessando o veio, uma ranhura ao longo dele, um rebaixe numa aresta.
joinery-how-mortise-tenon = Encoste a extremidade do travessão à outra peça (um travessão contra uma perna ou um montante).
joinery-how-butt = Encoste a extremidade ou a aresta de uma peça à outra.
joinery-how-dovetails = Encoste a extremidade de uma tábua contra o interior da outra junto à sua extremidade, como o lado de uma gaveta contra a frente. A tábua encostada fica com os rabos.
joinery-how-box-joint = Junte as extremidades das duas peças num canto (a frente e o lado de uma gaveta), encostadas ou sobrepostas.
joinery-how-half-lap = Cruze as duas tábuas de modo a sobreporem-se em toda a espessura.
joinery-how-overlap = Meta uma peça na outra pela profundidade da junta (uma prateleira 1/4" dentro do lado).

joinery-elsewhere = Outras cópias destas peças estão unidas ({ $joints }) onde se encontram noutros sítios; aqui ainda não.
joinery-already = Já unidas:
joinery-change = Alterar
joinery-change-hint = Tire-a e configure-a de novo a partir dos tamanhos
joinery-remove = Remover
joinery-remove-hint = Tire esta junta de ambas as peças
joinery-removed = Junta removida.
joinery-set-up-again = Configure-a de novo e depois faça-a.
joinery-add-another = Acrescentar outra junta
# What an unnamed part is called while describing a joint.
joinery-part-default = Peça
joinery-joint = Junta
joinery-place-first = { $joint } (coloque primeiro as peças)
joinery-joints = Juntas
joinery-at = (junto a { $at })
joinery-row = { $kind } com { $other }{ $at }, { $detail }

joinery-cut-into = Cortar { $cut } em { $into }
joinery-cut-into-hint = { $cut } mantém o tamanho; { $into } recebe o corte
joinery-cut-in = { $kind } cortado em { $into }.
joinery-make-half-lap = Fazer a meia-madeira
joinery-half-lap-made = Meia-madeira feita.
joinery-finger-width = Largura do dente
joinery-finger-width-hint = Arredondado, para que haja um número ímpar de dentes
joinery-make-box-joint = Fazer a junta de dentes
joinery-box-joint-made = Junta de dentes feita.
joinery-end-meets = A extremidade de { $part } encontra { $into }.

joinery-depth = Profundidade
joinery-depth-hint = Até onde entra; a junta cresce este valor
joinery-let-into = Encaixar { $part } em { $into }
joinery-let-in-done = { $kind } cortado em { $into }; { $part } mede { $length } a mais.

joinery-through = Passante
joinery-through-hint = Os rabos aparecem nas duas faces do canto
joinery-half-blind = Meio ocultas
joinery-half-blind-hint = Escondidas de frente, como numa frente de gaveta
joinery-tails = Rabos
# The three dovetail hints say what the word means first: plenty of people meet these names
# for the first time in this panel.
joinery-tails-hint = Quantos rabos cortar. Os rabos são as peças em leque nesta tábua, as que se veem ao longo do lado de uma gaveta; mais rabos, cada um mais estreito, é o aspeto de corte à mão.
joinery-pins = Pinos
joinery-pins-hint = Os pinos são as cunhas na outra tábua que preenchem os vãos entre os rabos. É a largura de cada um onde aparece: estreito (1/8" a 1/4") é corte à mão, tão largo como um rabo é corte à máquina.
joinery-half-pins = Meios-pinos
joinery-half-pins-hint = Os meios-pinos são os das arestas de cima e de baixo, com um rabo de um só lado. Aguentam o canto, por isso costumam ser um pouco mais largos do que os pinos entre os rabos.
joinery-dovetail-note = { $part } fica com os rabos, { $into } com os pinos (declive 1:8).
joinery-cut-dovetails = Cortar os rabos
joinery-dovetails-done = Rabos cortados; { $part } ganha o comprimento dos rabos.

joinery-spline-note = Um { $piece } ao longo de toda a junta, colado numa ranhura em cada peça.
joinery-cut-spline = Cortar a ranhura da chave
joinery-spline-done = Ranhura da chave cortada em ambas as peças.

joinery-size = Tamanho
joinery-count = Quantidade
joinery-count-hint-slots = Espalhadas uniformemente ao longo da junta
joinery-fit = Folga
joinery-fit-hint-slots = Quanto mais largas as cavidades são do que as peças (mais grossas, mais compridas, mais fundas); 0 para um ajuste justo
joinery-cut-biscuit-slots = Cortar as cavidades para biscuits
joinery-cut-loose-tenon-mortises = Cortar as caixas para espigas soltas
joinery-slots-done = { $count } cavidades cortadas em cada peça, para { $count } x { $piece }.

joinery-count-hint-pocket = Cerca de um a cada 4"
joinery-screw = Parafuso
joinery-screw-hint = Comprimento, para a espessura da peça que atravessam
joinery-pocket-note = Furado a partir da face oculta de { $part }; os furos não são desenhados.
joinery-add-pocket-screws = Acrescentar parafusos de bolso
joinery-pocket-done = { $count } parafusos de bolso acrescentados.

joinery-through-tenon = Espiga passante ({ $length } de comprimento, aparece do outro lado)
joinery-haunch = Haunch (enche uma ranhura de painel na extremidade)
joinery-length = Comprimento
joinery-length-hint = Até onde a espiga entra na caixa
joinery-thickness = Espessura
joinery-thickness-hint = Cerca de um terço do travessão é o habitual
joinery-shoulders = Ombros
joinery-shoulders-hint = Recuados das arestas do travessão, de cada lado
joinery-fit-hint-mortise = Quanto maior é a caixa do que a espiga (mais larga, mais grossa, mais funda); 0 para um ajuste justo
joinery-tenon-mortise = Espiga em { $part }, caixa em { $into }
joinery-mortise-tenon-done = Caixa e espiga feitas; { $part } mede { $length } a mais na lista de corte.

joinery-count-hint-dowels = Espalhadas uniformemente pela largura
joinery-diameter = Diâmetro
joinery-diameter-hint = 3/8" para stock de 3/4" é o habitual
joinery-depth-hint-dowels = Profundidade do furo em cada peça
joinery-fit-hint-dowels = Quanto maiores os furos são do que as cavilhas; 0 para um ajuste justo
joinery-drill-dowels = Furar para cavilhas
joinery-dowels-done = { $count } furos de cavilha furados em cada peça.

## Angled joints, where two parts meet at something other than a right angle

joinery-meet-angle = Encontram-se a { $angle }°.
joinery-draw-into = Desenhe a extremidade de uma peça dentro da outra onde vai a junta: a sobreposição define a profundidade.
joinery-angled-mortise-tenon-hint = Uma espiga pelo meio, numa caixa com a sua forma
joinery-housing = Alojamento
joinery-housing-hint = Toda a extremidade encaixada na outra peça
joinery-every-copy = Cortar todas as cópias
joinery-every-copy-hint = Estas são cópias de um componente partilhado: corte todas igualmente, ou só esta (passa a ser um componente próprio)
joinery-let-into-done = Encaixe de { $part } em { $into } feito.
joinery-angled-thickness-hint = Pelo meio da peça com espiga; cerca de um terço
joinery-angled-shoulders-hint = Recuados das arestas, de cada lado
joinery-angled-fit-hint = Quanto menor é a espiga do que a sua caixa (mais fina, mais estreita, mais curta); 0 para um ajuste justo
joinery-angled-done = Caixa e espiga em ângulo feitas: a extremidade de { $part } fica aparada com a face de { $into }, com a espiga dentro.

## The Cut List window

cutlist-title = Lista de Corte
cutlist-tab-parts = Peças
cutlist-tab-diagrams = Diagramas de corte
cutlist-tab-movement = Movimento da madeira
cutlist-tab-settings = Kerf da serra
cutlist-print-cut-sheet = Imprimir folha de corte...
cutlist-print-cut-sheet-hint = Guardar as peças e os diagramas de corte como uma página para imprimir, ou guardar como PDF, a partir do navegador
cutlist-print-build-sheets = Imprimir folhas de montagem...
cutlist-print-build-sheets-hint = Uma página por peça para a oficina: o seu tamanho, desenhos, juntas e perfis, por ordem
cutlist-export-csv = Exportar CSV...
cutlist-save-failed = Não foi possível guardar { $path }: { $error }
cutlist-saved-not-opened = Guardado, mas não foi possível abrir: { $error }
cutlist-kerf-note = O kerf que a serra tira em cada corte, guardado com o desenho.
cutlist-stock-note = Os tamanhos de stock (espessuras, tamanhos de painel, secções e comprimentos de madeira estrutural) definem-se por material no painel Materiais.
cutlist-saw-kerf = Kerf da serra
cutlist-kerf-unreadable = O kerf não se lê como um comprimento; usa-se o último bom.

cutlist-no-parts = Ainda não há peças. Cada grupo ou componente é uma peça: tábuas feitas com Empurrar/Puxar contam automaticamente.
# How many parts still have no material. The verb agrees with the number in English, which is
# why this is one message and not a sentence glued together from three.
cutlist-needs-material =
    { $count ->
        [one] { $warning } { $count } peça precisa de um material (listada primeiro). Pinte cada uma, ou pinte o grupo à volta delas.
       *[other] { $warning } { $count } peças precisam de um material (listadas primeiro). Pinte cada uma, ou pinte o grupo à volta delas.
    }
cutlist-loose-faces = { $count } faces soltas não estão em nenhum grupo, por isso não são peças. Agrupe-as (Ctrl+G) para as listar.
cutlist-needs-a-material = Precisa de um material
cutlist-parts-count =
    { $count ->
        [one] { $count } peça
       *[other] { $count } peças
    }
cutlist-material-line = { $kind }  ·  { $parts }  ·  { $amount }
cutlist-edit-material = Editar material
cutlist-square-feet = { $amount } ft²
cutlist-linear-feet = { $amount } ft lineares
cutlist-board-feet = { $amount } board ft
cutlist-pieces = { $count } peças

# The parts table's column headings.
cutlist-column-name = Nome
cutlist-column-qty = Qtd
cutlist-column-length = Comprimento
cutlist-column-width = Largura
cutlist-column-thickness = Espessura
cutlist-column-stock = Stock
cutlist-glued-up = { $stock }, colado a partir de { $boards }
cutlist-glued-up-hint = { $boards } tábuas de cerca de { $width } x { $length }, arestas aplainadas e coladas, depois cortadas a tamanho
cutlist-no-stock = nenhum
cutlist-hardware = Ferragens

cutlist-nothing-to-cut = Ainda não há peças para cortar.
cutlist-hardware-not-cut = As ferragens não se cortam de stock.
cutlist-paint-to-plan = Pinte estas peças com um material (e defina o tipo dele) para planear os cortes.
cutlist-type-not-cut = Este tipo de material não se corta de stock. Defina-o como painel, madeira maciça ou madeira estrutural.
cutlist-buy = Comprar:
cutlist-buy-line = { $count } × { $what }
cutlist-used = ({ $percent }% usado)
cutlist-part-problem = Peça { $label }: { $reason }
cutlist-stock-line = { $n }. { $description }  ·  { $percent }% usado
cutlist-not-from-stock = Não cortado de stock (tipo de material Outro): { $materials }.

## The Wood movement tab

movement-intro = A madeira maciça incha e encolhe contra o veio com a mudança das estações. O contraplacado e o MDF não.
movement-swing = Variação de humidade
movement-swing-hint = Quanto o teor de humidade da madeira muda ao longo de um ano: cerca de 4% em interiores (6% num inverno seco a 10% num verão húmido); mais num armazém ou numa divisão sem aquecimento
movement-unknown = { $info } Sem dados de movimento para { $materials }: defina «Move como» no painel Materiais para verificar as suas peças.
movement-none-known = Ainda não há peças de madeira maciça com espécie conhecida. Pinte peças com uma madeira da biblioteca, como Cherry ou Red Oak.
movement-no-conflicts = { $check } Nenhuma peça trava o movimento das outras.
movement-watch = Vigie estas juntas
movement-fastened-note = Se uma peça está fixa de modo a poder mover (botões de tampo, figure-8s, furos oblongos de parafuso), assinale «Fixado de modo a poder mover-se» nela, em Informação da Entidade.
movement-allowed = Admitido para
movement-how-much = Quanto cada peça move contra o veio
movement-column-part = Peça
movement-column-wood = Madeira
movement-column-width = Largura
movement-column-moves = Move cerca de
movement-assumed-flat-hint = O corte não está definido, por isso a peça é figurada como corte tangencial, que é o que move mais. Defina o corte em Informação da Entidade.

## The Start page

start-back-to = ‹  Voltar a { $drawing }
start-back-to-hint = Esc
start-title = Início
start-intro = Faça um desenho, abra um em que estava a trabalhar, ou desmonte um exemplo.
start-new-drawing = Novo desenho
start-inches = Polegadas
start-millimeters = Milímetros
start-open = Abrir...
start-open-under = Um desenho ou um modelo SketchUp (Ctrl+O)
start-import = Importar...
start-import-under = Um modelo SketchUp, para o desenho que está aberto
start-recent = Recentes
start-recent-empty = Os desenhos que abrir ou guardar aparecem aqui.
start-new = Novo
start-samples = Exemplos
start-samples-note = Peças acabadas, construídas de novo de cada vez que abre uma. Mude o que quiser: não são ficheiros, portanto nada aqui pode ser substituído.
# How a card reads out to a screen reader: its title, then the line under it.
start-card-label = { $title }. { $under }

## Opening, saving and the title bar

files-untitled = Sem título
files-save-failed = Não foi possível guardar { $path }: { $error }
files-open-failed = Não foi possível abrir { $path }: { $error }
files-export-failed = Não foi possível exportar { $path }: { $error }
# The window's title: the drawing's name, a star if there are unsaved changes, then the app.
files-window-title = { $name }{ $dirty } - WoodSketch
# Dismisses a message about a file that couldn't be opened or saved.
files-ok = OK
files-save-changes = Guardar alterações?
files-save-changes-question = Quer guardar as alterações a { $name }?
files-save = Guardar
files-dont-save = Não Guardar
files-cancel = Cancelar
# The named kinds of file in the system Open and Save dialogs.
files-filter-drawing = Desenho WoodSketch
files-filter-drawings-and-skp = Desenhos e modelos SketchUp
files-filter-sketchup = Modelo SketchUp (2021 ou posterior)
files-filter-web-page = Página web

## Report a Problem and Send Feedback

feedback-problem-title = Comunicar um Problema
feedback-idea-title = Enviar Comentários
feedback-unreachable = Não foi possível chegar a woodsketch.net ({ $error }). Verifique a ligação, ou use Abrir no GitHub.
feedback-after-crash = O WoodSketch fechou inesperadamente. O que eu estava a fazer:
feedback-thanks = { $check } Obrigado! É a issue #{ $number } no GitHub:
feedback-follow-email = Acompanhe-a lá para ver as respostas. Se houver perguntas, recebe um email.
feedback-follow = Acompanhe-a lá para ver as respostas (não precisa de conta para a ler).
feedback-close = Fechar
feedback-kind-problem = Algo correu mal
feedback-kind-idea = Uma ideia ou outro comentário
feedback-title-label = Título
feedback-what-happened = O que aconteceu? (o que fez, o que esperava)
feedback-your-idea = A sua ideia
feedback-email = O seu email (opcional)
feedback-email-hint = Só se quiser ser contactado sobre isto
feedback-email-note = Só o programador o vê, para perguntar sobre este relato. Não é publicado em lado nenhum.
feedback-include-counts = Incluir contagens do desenho aberto (sem nomes)
feedback-include-counts-hint = Quantas peças, faces, juntas e materiais de cada tipo: ajuda a reproduzir um problema
feedback-sent-with = Enviado com ele:
feedback-public-note = Torna-se uma issue pública no GitHub, onde a pode acompanhar. Não precisa de conta para a enviar.
feedback-bad-email = Esse endereço de email não parece válido.
feedback-send = Enviar
feedback-sending = A enviar...
feedback-open-github = Abrir no GitHub, em alternativa
feedback-open-github-hint = Submeta você mesmo no GitHub (precisa de uma conta gratuita); o seu email não é incluído
feedback-cancel = Cancelar

## Checking for updates (Help > Check for Updates)

updates-checking = A verificar atualizações...
updates-not-installed = Esta cópia não foi instalada com o instalador do WoodSketch, por isso não se atualiza a si própria. Descarregue a versão mais recente em woodsketch.net.
updates-up-to-date = O WoodSketch { $version } está atualizado.
updates-downloading = O WoodSketch { $version } está a descarregar em segundo plano (mostrado em baixo à direita). Quando estiver pronto, aparece aí um botão verde «Reiniciar para atualizar».
updates-ready = O WoodSketch { $version } está pronto: reinicie para atualizar.
updates-failed = Não foi possível verificar atualizações: { $error }

## While a sample is being built

building-title = A construir
building-what = A construir o { $sample }
building-note = É construído a partir de uma receita de cada vez, com as junções que a app tem agora.

## Opening and importing, crash reports, recovery, and the file-changed question

app-opened-skipped = Aberto, mas não foi possível ler { $count } faces, que ficaram de fora.
app-imported-skipped = Importado, mas não foi possível ler { $count } faces, que ficaram de fora.
app-skp-empty = Esse modelo SketchUp está vazio.
app-crash-title = O WoodSketch teve um problema
app-crash-note = Fechou inesperadamente da última vez ({ $version }). Um relato ajuda a corrigi-lo: diz o que correu mal e onde no código do WoodSketch, e vê tudo antes de ser enviado.
app-crash-report = Reportar...
app-crash-not-now = Agora não
# What a drawing with no file of its own is called in the question below.
app-the-drawing = O desenho
app-file-changed-title = O ficheiro mudou fora do WoodSketch
app-file-changed = { $name } foi alterado por outra coisa, e este desenho tem alterações por guardar. Só um dos dois pode ser mantido.
app-reload = Recarregar do ficheiro
app-reload-hint = Tomar o ficheiro como está agora, perdendo as alterações por guardar aqui (Ctrl+Z as traz de volta)
app-keep-mine = Manter o meu
app-keep-mine-hint = Deixar este desenho como está; guardar sobrepõe o ficheiro
app-recover-title = Recuperar trabalho não guardado?
app-recover-note = O WoodSketch não fechou corretamente da última vez. Guardou uma cópia das suas alterações não guardadas:
app-recover-minutes = , há { $minutes } minutos
app-recover-hours = , há { $hours } horas
app-recover-line = { $name }{ $when }
app-recover = Recuperar
app-discard = Descartar
app-discard-all = Descartar tudo
app-build-sheets-title = Folhas de montagem de { $name }
app-autosave-failed = Não foi possível guardar automaticamente: { $error }
# The headings on the printable pages, which open in a browser.
cutlist-sheet-title = Lista de corte
cutlist-build-sheets-title = Folhas de montagem

## The status bar's left-hand side: what the tool in your hand is waiting for.
##
## These are read while working, so they are terse on purpose, and they name keys (Ctrl, Esc,
## W A S D) that are the same on every keyboard. Keep the key, translate around it.

status-pick-reference = Clique na face de referência da peça (uma face larga), aresta (uma aresta comprida) ou extremidade. Esc termina.
status-select = Selecione objetos. Ctrl acrescenta, Shift alterna, Ctrl+Shift remove. Arraste para a direita para janela, para a esquerda para seleção cruzada.
status-orbit = Arraste para orbitar.
status-pan = Arraste para deslocar a vista.
status-zoom = Arraste para cima para aproximar, para baixo para afastar.
status-walk = A andar: mova o rato para olhar; W A S D ou as setas andam (Shift: mais depressa), E e Q sobem e descem, F voa, a roda define a velocidade; escreva uma altura dos olhos. As paredes não o param; o chão o segura. Esc termina.

# What the Measurements box at the bottom right is asking for.
measure-dimensions = Dimensões
measure-angle = Ângulo
measure-length = Comprimento
measure-radius = Raio
measure-bulge = Arqueamento
measure-scale = Escala
measure-distance = Distância
measure-text = Texto
measure-eye-height = Altura dos olhos
measure-measurements = Medições

# Inference: which axis the cursor has lined up with. The colors are the drawing's own -- red is
# X, green is Y, blue is Z, as in SketchUp -- so they are names here, not just colors.
infer-on-red = No Eixo Vermelho
infer-on-green = No Eixo Verde
infer-on-blue = No Eixo Azul
infer-parallel = Paralelo à Aresta
infer-perpendicular = Perpendicular à Aresta

tools-part-gone = Essa peça desapareceu.
tools-click-part = Clique na própria peça.
tools-click-part-reference = Clique na própria peça para escolher a sua face ou aresta de referência.
tools-opened-group = Aberto { $name } para trabalhar lá dentro. Esc ou clique fora fecha.
tools-select-to-group = Selecione algo para criar um { $noun }.
tools-noun-group = grupo
tools-noun-component = componente
tools-no-guides = Não há guias para eliminar.
tools-select-to-explode = Selecione um grupo ou componente para explodir.
tools-select-to-copy = Selecione algo para copiar
tools-nothing-to-paste = Nada para colar: copie algo primeiro

## What each tool says while you use it, and what it says when it can't do what was asked.
##
## `[x,y,z]`, `3x`, `/3`, `24s` and `5r` are typed into the Measurements box and are the same in
## every language: keep them exactly, and translate the words around them.

# Snapping: what the point under the cursor is.
snap-endpoint = Ponto final
snap-origin = Origem
snap-guide-point = Ponto-guia
snap-intersection = Interseção
snap-midpoint = Ponto médio
snap-on-edge = Na aresta
snap-on-guide = No guia
snap-on-face = Na face

# Arc (two points and a bulge)
tool-arc-start = Selecione o ponto inicial (ou escreva [x,y,z]). Lados: { $sides } (escreva 24s para mudar).
tool-arc-end = Selecione o ponto final ou escreva o comprimento da corda.
tool-arc-bulge = Selecione o arqueamento, ou escreva-o (5r para um raio).
tool-arc-click-start-end = Clique no início do arco, depois no fim
tool-arc-radius-min = O raio tem de ser pelo menos metade da corda ({ $half })
tool-arc-end-away = Escolha um ponto final afastado do início

# Center arc, and the three-point arc
tool-arc-center = Selecione o centro. Lados: { $sides } (escreva 24s para mudar). As setas deitam o arco num eixo.
tool-arc-where-starts = Selecione onde o arco começa, ou escreva o raio.
tool-arc-turn-and-click = Rode e clique onde acaba, ou escreva o ângulo.
tool-arc-start-away = Escolha um ponto inicial afastado do centro
tool-arc-click-center-first = Clique primeiro no centro
tool-arc-turn-away = Rode para o lado oposto ao início para desenhar o arco
tool-arc-three-first = Selecione o início do arco. Lados: { $sides } (escreva 24s para mudar).
tool-arc-three-second = Selecione um ponto por onde o arco passa.
tool-arc-three-third = Selecione o fim do arco.
tool-arc-second-away = Escolha um segundo ponto afastado do primeiro
tool-arc-in-a-line = Os três pontos estão em linha: escolha um fim fora dela
tool-arc-three-points = Clique em três pontos: o início, um no arco, o fim. Escreva 24s para 24 lados.

# Circle and Polygon
tool-circle-sides = Os lados têm de ser um número inteiro de 3 a 999, como 24s
tool-circle-center-then-radius = Clique no centro, depois escreva o raio
tool-circle-corner = Selecione um vértice do círculo ou escreva o raio. Lados: { $sides } (escreva 8s para mudar).
tool-polygon-corner = Selecione um vértice do polígono ou escreva o raio. Lados: { $sides } (escreva 8s para mudar).
tool-circle-center = Selecione o ponto central (ou escreva [x,y,z]). Lados: { $sides } (escreva 32s para mudar). As setas orientam-no num eixo.
tool-polygon-center-to-corner = Selecione o ponto central (ou escreva [x,y,z]). Lados: { $sides } (escreva 8s para mudar). As setas orientam-no num eixo. Raio até um vértice (Ctrl: até um lado).
tool-polygon-center-to-side = Selecione o ponto central (ou escreva [x,y,z]). Lados: { $sides } (escreva 8s para mudar). As setas orientam-no num eixo. Raio até ao meio de um lado (Ctrl: até um vértice).
tool-circle-through-point = Por um ponto
tool-circle-move-away = Afaste-se do centro, ou escreva o raio
tool-circle-radius-positive = O raio tem de ser maior que zero

# Eraser, Line, Offset, Paint Bucket
tool-eraser-status = Clique ou arraste sobre arestas, guias ou grupos para os apagar.
tool-eraser-click = Clique numa aresta, guia, grupo ou componente para o apagar
tool-line-start = Selecione o ponto inicial, ou escreva [x,y,z].
tool-line-end = Selecione o ponto final ou escreva um valor. Mantenha Shift ou prima uma seta para travar uma direção; Esc termina a linha.
tool-line-length-positive = O comprimento tem de ser maior que zero
tool-line-click-start = Clique no ponto inicial (ou escreva [x,y,z]), depois escreva o comprimento
tool-offset-pick = Clique numa face para compensar o contorno dela (ou selecione primeiro uma face).
tool-offset-move = Mova para dentro ou para fora e clique, ou escreva a distância de compensação.
tool-offset-click-face = Clique numa face para compensar o contorno dela
tool-offset-click-then-type = Clique numa face, depois escreva a compensação
tool-offset-move-first = Mova primeiro o rato para dentro ou para fora da face
tool-paint-status = Clique numa tábua para a pintar toda com o material escolhido no painel Materiais. Ctrl-clique: uma face; Alt-clique: colher amostra.
tool-paint-click = Clique num grupo, componente ou face para o pintar.

# Push/Pull
tool-pushpull-pick = Escolha uma face para empurrar ou puxar. Ctrl mantém a face. Clique duplo numa face repete a última distância.
tool-pushpull-keeping-pick = A manter a face (Ctrl): escolha uma face; uma face nova e paredes saem dela.
tool-pushpull-move = Mova para definir a distância, depois clique, ou escreva um valor. Ctrl mantém a face.
tool-pushpull-keeping = A manter a face: uma face nova e paredes. Mova para definir a distância, depois clique, ou escreva um valor.
tool-pushpull-click-then-type = Clique numa face, depois escreva a distância
tool-pushpull-standard-thickness = Espessura normal
tool-pushpull-in-group = Essa face faz parte de um grupo. Clique duplo no grupo para editar lá dentro, depois empurre/puxe.

# Rectangle, and the rotated rectangle
tool-rect-first = Selecione o primeiro canto, ou escreva [x,y,z].
tool-rect-opposite = Selecione o canto oposto ou escreva um valor.
tool-rect-square = Quadrado
tool-rect-golden = Secção áurea
tool-rect-enter-wh = Escreva largura e altura, como 1,5";3,5"
tool-rect-dims-positive = As dimensões têm de ser maiores que zero
tool-rect-click-first = Clique no primeiro canto, depois escreva o tamanho
tool-rotrect-first = Selecione o primeiro canto.
tool-rotrect-side = Selecione o segundo canto (o primeiro lado), ou escreva o comprimento dele.
tool-rotrect-across = Mova de lado e clique, ou escreva o comprimento do outro lado.
tool-rotrect-second-away = Escolha o segundo canto afastado do primeiro
tool-rotrect-click-first = Clique no primeiro canto
tool-rotrect-move-across = Mova transversalmente ao primeiro lado para dar a largura ao retângulo

# Scale
tool-scale-pick-grip = Clique numa pega: os cantos escalam por igual, as arestas em duas direções, as faces numa.
tool-scale-move-center = Mova para escalar, ou escreva um fator (2, 0,5) ou um tamanho (36"). Pelo centro (Ctrl fixa o lado oposto). Mantenha Shift para alternar uniforme e esticado.
tool-scale-move-opposite = Mova para escalar, ou escreva um fator (2, 0,5) ou um tamanho (36"). Ctrl escala pelo centro. Mantenha Shift para alternar uniforme e esticado.
tool-scale-click-grip = Clique numa pega de escala, depois escreva a escala
tool-scale-hover-grips = Clique numa das pegas de escala (paire sobre algo para as ver)
tool-scale-positive = A escala tem de ser maior que zero (o espelhamento ainda não é suportado)
# $value is what was typed, kept as it was typed.
tool-scale-not-a-factor = "{ $value }" não é um fator de escala
tool-scale-one-or-n = Escreva um valor, ou { $count } separados por vírgulas

# Ctrl+D with nothing picked.
tool-nothing-to-duplicate = Selecione algo para duplicar

# Tape Measure
tape-note-guide-line = + Linha-guia
tape-note-guide-point = + Ponto-guia
tape-note-measure = Medir
tool-tape-guide-lines = Criar Linhas-guia: clique numa aresta para tirar um guia dela, ou num ponto. Ctrl muda de modo.
tool-tape-guide-points = Criar Pontos-guia: clique de onde medir. Ctrl muda de modo.
tool-tape-measure-mode = Medir: clique de onde medir, ou paire sobre uma aresta. Escreva um comprimento depois de medir para redimensionar o modelo. Ctrl muda de modo.
tool-tape-move-away = Afaste-se da aresta; clique ou escreva o desvio do guia.
tool-tape-click-to = Clique no ponto até onde medir.
tool-tape-click-to-or-type = Clique no ponto até onde medir, ou escreva a distância.
tool-tape-show-direction = Mova o rato para mostrar a direção, depois escreva a distância
tool-tape-measure-first = Meça uma distância, depois escreva o comprimento que devia ter
tool-tape-distance-positive = A distância tem de ser maior que zero
tool-tape-click-from = Clique de onde medir, depois escreva a distância

# Dimensions and text notes
tool-dim-first = Escolha o primeiro ponto de onde medir, ou clique numa aresta para a cotar.
tool-dim-second = Escolha o segundo ponto.
tool-dim-place = Mova para fora para colocar a cota, depois clique, ou escreva quanto ela sai.
tool-dim-second-away = Escolha um segundo ponto afastado do primeiro
tool-dim-two-points = Escolha dois pontos, depois escreva quanto sai a cota
tool-note-pick-point = Escolha o ponto a que a nota se refere.
tool-note-where-words = Clique onde vão as palavras (no mesmo sítio: sem linha guia).
tool-note-type-words = Escreva as palavras da nota e prima Enter, ou escolha um ponto para outra nota.
tool-note-place-first = Coloque uma nota (clique num ponto, depois onde vão as palavras), a seguir escreva-as

# Move. The copying ones are whole sentences rather than a sentence with "(copying)" dropped
# into it, because where that note belongs is a question about the language.
tool-move-base = Escolha um ponto de base para mover (ou escreva [x,y,z]). Ctrl copia; escreva 3x ou /3 depois de uma cópia para uma série.
tool-move-base-copying = Escolha um ponto de base para mover (a copiar) (ou escreva [x,y,z]). Ctrl copia; escreva 3x ou /3 depois de uma cópia para uma série.
tool-move-destination = Escolha o destino, ou escreva uma distância. As setas travam um eixo; Ctrl copia.
tool-move-destination-copying = Escolha o destino, ou escreva uma distância (a copiar). As setas travam um eixo; Ctrl copia.
tool-move-pasting = Clique para largar o que foi colado (liga-se a pontos), ou escreva uma distância. Esc retira-o.
tool-move-copy-first = Copie algo primeiro (Ctrl enquanto move), depois escreva 3x ou /3
tool-move-arrays-need-copy = As séries precisam de uma cópia: Ctrl enquanto move
tool-move-click-base = Clique num ponto de base, depois escreva a distância
tool-move-select = Selecione algo para mover, ou comece sobre ele
tool-move-away-from-base = Afaste primeiro o rato do ponto de base
tool-move-vector = Escreva um vetor como x,y,z
tool-move-show-direction = Mova o rato para mostrar a direção, depois escreva a distância

# Rotate, and the Protractor
tool-rotate-center = Escolha o centro de rotação. As setas travam o transferidor (Direita vermelho, Esquerda verde, Cima azul); Ctrl copia.
tool-rotate-center-copying = Escolha o centro de rotação (a copiar). As setas travam o transferidor (Direita vermelho, Esquerda verde, Cima azul); Ctrl copia.
tool-rotate-start = Escolha um ponto para começar a rotação.
tool-rotate-start-copying = Escolha um ponto para começar a rotação (a copiar).
tool-rotate-end = Escolha o fim da rotação, ou escreva um ângulo.
tool-rotate-end-copying = Escolha o fim da rotação, ou escreva um ângulo (a copiar).
tool-protractor-center = Escolha o centro do transferidor (por ele passa uma linha-guia). As setas travam-no (Direita vermelho, Esquerda verde, Cima azul).
tool-protractor-start = Escolha um ponto para a direção inicial.
tool-protractor-direction = Escolha a direção do guia, ou escreva um ângulo.
tool-rotate-away-from-center = Escolha um ponto afastado do centro
tool-rotate-copy-first = Rode primeiro uma cópia (Ctrl enquanto roda), depois escreva 3x ou /3
tool-rotate-arrays-need-copy = As séries precisam de uma cópia: Ctrl enquanto roda
tool-rotate-pick-start-dir = Escolha a direção inicial, depois escreva o ângulo
tool-rotate-pick-center-dir = Escolha um centro e uma direção inicial, depois escreva o ângulo
tool-rotate-select = Selecione algo para rodar, ou comece sobre ele
tool-rotate-turn-away = Rode primeiro para longe da direção inicial
tool-rotate-invalid-angle = Ângulo inválido: "{ $typed }"

# The Board tool. $stock is the material and size picked in the Board panel ("Pine Lumber 2x4").
tool-board-no-material = Tábua: acrescente um material de madeira (painel Materiais, +) para desenhar tábuas do stock dele.
tool-board-lumber-start = { $stock }: clique onde a tábua começa. Escreva 2x4x96, ou 3,6 1/2 para um tamanho personalizado. Ctrl põe-na de canto.
tool-board-sheet-start = { $stock }: clique num canto da tábua.
tool-board-lumber-end = Clique onde acaba, ou escreva o comprimento. As setas travam um eixo. W espelha a tábua.
tool-board-sheet-end = Clique no canto oposto, ou escreva comprimento,largura. W espelha a tábua.
tool-board-add-material = Acrescente primeiro um material de madeira (painel Materiais, +), depois escolha o stock dele no painel Tábua.
tool-board-length-first = Escolha primeiro um comprimento: afaste-se do início.
tool-board-no-size = Essa tábua não tem tamanho.
tool-board-pick-stock = Escolha primeiro stock no painel Tábua.
tool-board-click-start = Clique onde a tábua começa, depois escreva o comprimento (ou escreva 2x4x96).
tool-board-length-positive = O comprimento tem de ser maior que zero.
tool-board-show-direction = Mova o rato no sentido em que a tábua deve correr, depois escreva o comprimento.
tool-board-length-width = Escreva comprimento e largura, como 30,11 1/4
tool-board-no-thickness = Esse tamanho não tem espessura nem largura.
tool-board-pick-sheet = Escolha primeiro um material de painel ou madeira maciça no painel Tábua.

## The viewport: what is drawn over the drawing, and the right-click menu

viewport-editing = A editar  { $path }    Esc ou clique fora para fechar
viewport-walk-speed = Velocidade a andar: { $speed } por segundo
viewport-flying = A voar: nada o segura. E e Q sobem e descem; F volta a andar.
viewport-walking = A andar: de volta ao chão.
viewport-grain = Veio
viewport-reference-face = Face de referência
viewport-reference-edge = Aresta de referência
viewport-from-point = A partir do ponto
viewport-make-unique = Tornar Único
viewport-make-unique-hint = A sua própria cópia, para que mudá-lo não mexa nas outras
viewport-reverse-faces = Inverter Faces
viewport-reverse-faces-hint = Virar as faces selecionadas, frente para trás
