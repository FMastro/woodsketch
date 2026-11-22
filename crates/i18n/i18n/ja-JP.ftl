# WoodSketch 日本語カタログ。
# is the one the code asks for, and a key missing from a translation shows in English rather
# than showing nothing.
#
# A key is `area-thing`, and a tooltip is the same key with `-hint`. Keep a key when its wording
# changes and the translations keep up by themselves; change a key only when the thing it names
# has become something else, so that a stale translation is noticed rather than silently kept.
#
# Punctuation belongs to the translation. A trailing `...` means "this opens something that will
# ask you more", which is a Windows convention a translator should be free to drop.

## The menu bar

menu-file = ファイル
menu-edit = 編集
menu-view = 表示
menu-camera = カメラ
menu-draw = 描画
menu-tools = ツール
menu-window = ウィンドウ
menu-help = ヘルプ

## File

file-start-page = スタートページ
file-start-page-hint = 新規、開く、最近使った項目とサンプル。Escで戻ります
file-new = 新規
file-open = 開く…
file-open-recent = 最近使ったファイル
file-save = 保存
file-save-as = 名前を付けて保存…
file-import-skp = SketchUpモデルを読み込む…
file-import-skp-hint = .skp（SketchUp 2021以降）をこの図面に追加
file-print-build-sheets = 作業工程表を印刷…
file-print-build-sheets-hint = 部品ごとの工程と組み立て手順を作業工程表にして、印刷用にブラウザで開きます
file-export = 書き出し
file-export-obj = OBJ…
file-export-obj-hint = 材料色付きの3Dモデル。レンダラーや他のモデラー用
file-export-stl = STL…
file-export-stl-hint = 3Dプリント用の3Dモデル
file-export-dxf = CNC用DXF…
file-export-dxf-hint = 各部品を正面から見た平面図。輪郭、貫通穴、深さごとのポケットを含みます
file-export-svg = CNC用SVG…
file-export-svg-hint = 各部品を正面から見た平面図。輪郭は黒、穴は白、ポケットは灰色です
file-export-units = 単位
file-export-inches = インチ
file-export-millimeters = ミリメートル
file-exit = 終了

## Edit

edit-undo = 元に戻す
edit-redo = やり直す
edit-cut = 切り取り
edit-copy = コピー
edit-paste = 貼り付け
edit-paste-hint = マウスに追従します。クリックして配置します
edit-paste-in-place = 同じ位置に貼り付け
edit-duplicate = 複製
edit-duplicate-hint = すぐ隣に複製します
edit-delete = 削除
edit-delete-guides = ガイドを削除
edit-select-all = すべて選択
edit-select-none = 選択解除
edit-make-group = グループを作成
edit-make-component = コンポーネントを作成…
edit-explode = 分解
edit-flip-along = 反転方向
edit-flip-along-hint = 選択範囲をその場で鏡像反転します。グループまたはコンポーネントは自身の軸に沿って反転します
edit-flip-red = 赤方向
edit-flip-green = 緑方向
edit-flip-blue = 青方向

## View

view-grid = グリッド
view-wood-movement = 木材の伸縮
view-wood-movement-hint = 木目を横切る無垢材部品の伸縮量。接合部に拘束される箇所は赤、自由な箇所は緑で表示します
view-theme = テーマ
view-theme-light = ライト
view-theme-dark = ダーク
view-theme-system = システムに合わせる
view-theme-hint = ダークは夜間に目が疲れにくく、システムに合わせるとコンピューターの設定に従います
view-units = 単位
view-units-hint = 長さの表示方法と、単位なしで入力した数値の意味。図面と一緒に保存されます
view-guides = ガイド
view-guides-hint = すべてのガイドを表示または非表示にします（非表示のガイドにはスナップしません）
view-dimensions = 寸法
view-dimensions-hint = すべての寸法を表示または非表示にします
view-text = テキスト
view-text-hint = すべての注記を表示または非表示にします
view-language = 言語
view-language-hint = WoodSketchの表示言語。図面は変更されません

## Camera

camera-standard-views = 標準ビュー
camera-view-iso = アイソメトリック
camera-view-top = 上面
camera-view-front = 正面
camera-view-right = 右面
# The toolbar's view buttons, whose tooltip names the view: "Top view".
camera-view-hint = { $view }ビュー
camera-zoom-extents = 全体を表示

## Tools

tools-cut-list = 木取り表…

## Window

window-default-tray = 既定のトレイ
window-show-tray = トレイを表示
window-show-tray-hint = 右側にあるトレイ全体
window-panels-show-all = すべてのパネルを表示
window-panels-hide-all = すべてのパネルを隠す
window-panels-automatic = 自動
window-panels-automatic-hint = 表示する内容があるパネルだけを表示します。起動時はこの状態です
window-cut-list = 木取り表

## Help

help-check-for-updates = 更新を確認…
help-check-at-startup = 起動時に更新を確認
help-check-at-startup-hint = 起動時にGitHubのWoodSketchリリースを確認し、新しいバージョンをバックグラウンドでダウンロードします
help-whats-new = 更新内容…
help-whats-new-hint = 最近の更新内容を新しい順に表示します
help-user-guide = ユーザーガイド
help-user-guide-hint = WoodSketchの使い方と木工の基礎。F1で現在のツールのページを開きます
help-release-notes = リリースノート
help-release-notes-hint = GitHubにある各バージョンの更新内容
help-report-problem = 問題を報告…
help-report-problem-hint = 問題の内容を知らせます。役立つ詳細を含むGitHub issueを開きます。個人情報は含めません
help-send-feedback = フィードバックを送信…
help-send-feedback-hint = アイデアなどを送信します。GitHub issueを開きます
help-problems-and-ideas = 問題とアイデア
help-problems-and-ideas-hint = 報告・提案された内容と修正済みの問題をGitHubで確認します
help-discussions = ディスカッション
help-discussions-hint = 操作方法を質問したり、次に対応してほしい機能や作品を共有できます
help-about = WoodSketchについて

## The About window

about-title = WoodSketchについて
about-version = WoodSketch { $version }
about-whats-new = 更新内容
about-what-it-is = SketchUpのような操作感の、木工向け軽量3Dモデラーです。
about-not-affiliated = Trimbleとは提携・関係のない製品です。

## The status bar

status-reloaded = 再読み込み：WoodSketchの外でファイルが変更されました
status-reloaded-hint = Ctrl+Zで再読み込み前の図面に戻せます
status-dismiss = 閉じる
status-whats-new = 更新内容
status-whats-new-hint = 今回とそれ以前の更新内容
status-updated = { $version }に更新しました
status-downloading = WoodSketch { $version }をダウンロード中…
status-restart-to-update = 再起動して{ $version }に更新
status-restart-to-update-hint = 新しいバージョンをインストールしてWoodSketchを再び開きます（先に保存を求められます）

## The tools' own names, shown in the Tools menu and in each toolbar button's tooltip.
##
## These are SketchUp's names, because most people arriving here already know them from there.
## SketchUp has its own translation, so if it names a tool in your language, that name is the one
## your readers will be looking for -- use it rather than a fresh translation of the English.
## Leaving one in English is a reasonable choice too, if that is what people in your language
## actually say.

tool-name-select = 選択
tool-name-eraser = 消しゴム
tool-name-paint-bucket = ペイントバケツ
tool-name-line = 線
tool-name-arc = 円弧
tool-name-rectangle = 長方形
tool-name-circle = 円
tool-name-polygon = 多角形
tool-name-center-arc = 中心円弧
tool-name-three-point-arc = 3点円弧
tool-name-pie = 扇形
tool-name-rotated-rectangle = 回転した長方形
tool-name-board = 板材
tool-name-push-pull = 押し出し
tool-name-move = 移動
tool-name-rotate = 回転
tool-name-scale = 拡大縮小
tool-name-offset = オフセット
tool-name-tape-measure = メジャー
tool-name-protractor = 分度器
tool-name-dimension = 寸法
tool-name-text = テキスト
tool-name-orbit = オービット
tool-name-pan = パン
tool-name-zoom = ズーム
tool-name-walk = ウォーク

# A toolbar button's tooltip: the tool's name and the key that picks it. $key is the same on
# every keyboard, so keep it as it is.
tool-hint = { $tool }（{ $key }）
# The same, for a tool that isn't built yet.
tool-hint-coming-soon = { $tool }（{ $key }）：近日対応
# Said at the bottom of the window when someone picks a tool that isn't built yet.
tool-coming-soon = { $tool }（{ $key }）は近日対応です。
# Said when a length or an angle is typed while holding a tool that has nothing to do with one.
tool-takes-no-typed-values = { $tool }は数値入力に対応していません。

## The toolbar. Each of these is a tooltip, and carries the keyboard shortcut in brackets; the
## shortcut itself is the same on every keyboard, so leave it as it is and translate around it.

toolbar-new-hint = 新規（Ctrl+N）
toolbar-open-hint = 開く（Ctrl+O）
toolbar-save-hint = 保存（Ctrl+S）
toolbar-undo-hint = 元に戻す（Ctrl+Z）
toolbar-redo-hint = やり直す（Ctrl+Y）
toolbar-zoom-extents-hint = 全体を表示（Shift+Z）
toolbar-cut-list-hint = 木取り表：部品と切断図

## The Default Tray, down the right-hand side, and the name of each panel in it

tray-title = 既定のトレイ
tray-hide = トレイを隠す
tray-hide-hint = トレイを隠します（ウィンドウ > 既定のトレイ）
# The ✕ on a panel's header bar: named after the panel, since every panel has one.
tray-close-panel = { $panel }を閉じる
tray-panel-help = { $panel }のヘルプ
tray-panel-help-hint = このパネルのユーザーガイドを開きます
tray-close-panel-hint = このパネルを閉じます。ウィンドウ > 既定のトレイで戻せます
tray-no-panels = すべてのパネルが閉じています。ウィンドウ > 既定のトレイで戻せます
tray-board = 板材
tray-entity-info = エンティティ情報
tray-joinery = 接合
tray-edge-profile = エッジプロファイル
tray-templates = テンプレート
tray-materials = 材料
tray-tags = タグ
tray-outliner = アウトライナー
# The sizes of whichever template is selected: "Cabinet sizes", "Drawer sizes". The template's own
# name is not translated yet, so it arrives in English.
tray-template-sizes = { $template }のサイズ
# The same panel, named without a template to name: on its own in the Window menu, and on its
# header while nothing made from a template is selected.
tray-template-sizes-name = サイズ
# Said in a panel's body when it is open but has nothing to work on yet.
tray-joinery-needs = 接している2つの部品を選択して接合します
tray-template-sizes-needs = テンプレートまたはレシピから作成した部品を選択してサイズを変更します
tray-edge-profile-needs = 部品を1つ選択してエッジを加工します

## The What's New window (Help > What's New), shown once after an update

whats-new-title = WoodSketchの更新内容
# Marked beside the version that is running.
whats-new-this-one = （今回の更新）
# How many versions have arrived since the one that ran last. Two messages rather than one with a
# plural, because the singular drops the number altogether: "One update since 0.1.64:".
whats-new-recent = 最近の更新：
# "Done" rather than "Close", because clicking it marks the notes as read.
whats-new-done = 完了
whats-new-done-hint = 既読として記録します
whats-new-every-version = GitHubの全バージョン

## The Outliner panel: where you are in the drawing, and what is at this level

outliner-empty = グループまたはコンポーネントはまだありません。
outliner-group = グループ
outliner-group-hint = グループを作成（Ctrl+G）
outliner-component = コンポーネント
outliner-component-hint = コンポーネントを作成（G）
outliner-explode-hint = 選択したグループをばらした形状に戻します
outliner-auto-group = 新しい板材を自動的にグループ化
outliner-auto-group-hint = 押し出しで独立した面をソリッドにするとき、グループ化します
# The start of the trail showing which group is open: "Model > Case > Leg".
outliner-model = モデル
outliner-close-group = グループを閉じる（Esc）
# Marks a row whose tag is hidden, after the name.
outliner-hidden = （非表示）
outliner-open-hint = 開いています。この中を編集しています
outliner-needs-material-hint = 木取り表には材料が必要です

## The Board panel: the stock a new board is drawn from

board-material = 材料
board-pick-material = 材料を選択
# A material with what it comes as, when its name doesn't already say: "Pine (boards)".
board-with-noun = { $name }（{ $noun }）
board-noun-lumber = 製材
board-noun-boards = 板材
board-noun-sheets = シート材
# The headings the material dropdown groups by.
board-kind-lumber = 製材：2x4、2x6…
board-kind-boards = 板材：厚さ3/4インチ、1インチ…
board-kind-sheets = シート材：合板、MDF
board-from-library-hint = ライブラリから図面に追加
board-stock = 材料寸法
board-custom-size-open = カスタム寸法…
board-custom-size-open-hint = この材料の寸法を追加（または測定ボックスに入力）
board-lay-it = 配置
board-flat = 平置き
board-on-edge = 縦置き
board-on-edge-hint = 描画中にCtrlで切り替え
board-lumber-note = 製材は呼び寸法で表示され、実寸が括弧内に示されます。任意の長さで描画でき、木取り表が購入する板材を選びます。
board-sheet-note = この厚さの長方形として描画します。2x4などには製材材料を選択してください
board-custom-size = カスタム寸法
board-thickness = 厚さ
board-width = 幅
board-add-to = { $material }に追加
board-cancel = キャンセル
board-custom-note = 材料の寸法に追加され、木取り表で使用されます

## The Tags panel

tags-add-hint = タグを追加
tags-add-folder-hint = フォルダーを追加（タグを右クリックするとその中へ移動できます）
tags-untagged = タグなし
tags-fold = 折りたたむ
tags-unfold = 展開
# Two whole sentences rather than one built from pieces: a sentence assembled out of fragments
# only ever comes out right in the language it was assembled for.
tags-hide-folder = このフォルダー内をすべて非表示
tags-hide-tag = このタグの付いたものをすべて非表示
tags-show = 再表示
tags-rename-hint = ダブルクリックで名前を変更。行をフォルダーへドラッグ（または右クリック）すると移動できます
tags-delete-folder-hint = フォルダーを削除（中身は上の階層へ移動します）
tags-delete-tag-hint = タグを削除（グループは「タグなし」になります）
tags-move-to = 移動先
tags-top-level = 最上位
tags-no-folders = フォルダーはまだありません。フォルダーボタンで追加できます。

## Entity Info: what is selected, its name, tag and material

entity-nothing-selected = 何も選択されていません。
entity-name = 名前
entity-text = テキスト
entity-tag = タグ
entity-dimension = 寸法  { $length }
# Loose geometry — edges, faces, guides — rather than groups.
entity-loose =
    { $count ->
        [one] { $count }個の未グループ化要素
       *[other] { $count }個の未グループ化要素
    }
entity-selected = { $count }件を選択（グループ／コンポーネント { $groups }件）
entity-tags-note = タグはグループとコンポーネントに付きます。まずこれをグループ化してください（Ctrl+G）。
entity-material = 材料
entity-material-default = 既定

## The Materials panel, and the editor behind the pencil

materials-add-hint = 材料を追加
materials-none = 材料なし（既定）
materials-none-hint = 標準の簡素な外観です。これで塗装すると塗装を解除します。編集できる材料ではありません。＋で材料を追加してください。
# $kind is the sort of material (Solid wood, Sheet good); $pencil is the pencil icon.
materials-paint-hint = { $kind }。クリックするとこれで塗装します。{ $pencil }で編集します。
materials-edit-hint = この材料を編集
materials-empty = 材料はまだありません。{ $plus }をクリックして、レッドオークや合板などを追加してください。
materials-none-selected = 材料が選択されていません。塗装すると塗装を解除します。上の材料を選んでください。
materials-editing = { $name }を編集中
materials-close-editor = エディターを閉じる
materials-library = ライブラリ
materials-new-blank = 空の材料を新規作成
materials-name = 名前
materials-color = 色
materials-type = 種類
materials-grain = 木目
materials-has-grain = 木目あり
materials-has-grain-hint = 切断時は部品を木目方向に合わせる必要があります
materials-thicknesses = 厚さ
materials-sheet-sizes = 板材サイズ
materials-board-lengths = 板材の長さ
materials-board-widths = 板材の幅
materials-lengths = 長さ
materials-not-cut = 材料から切り出さない部品です。木取り表では部品として数えます。
materials-moves-like = 伸縮特性
materials-moves-like-hint = 木材の伸縮チェックで使う、季節による伸縮が同じ樹種
materials-movement-custom = カスタム
materials-movement-unknown = 不明
materials-sections = サイズ区分（{ $total }件中{ $shown }件）
materials-sections-note = 取り扱う公称寸法と実寸法です。各部品には収まる最小サイズが割り当てられます。
materials-delete = 材料を削除
materials-delete-hint = 塗装したものを材料なしに戻します

## Which way a face or an edge points in the drawing.
##
## Two forms of each, because English puts them in sentences two ways: "Top-left" starts a name,
## "around the top" sits inside one. A language that doesn't change the word can give both the
## same text; one that changes it more than English does has somewhere to put each form.

side-top = 上
side-bottom = 下
side-left = 左
side-right = 右
side-front = 正面
side-back = 背面
side-top-lower = 上
side-bottom-lower = 下
side-left-lower = 左
side-right-lower = 右
side-front-lower = 正面
side-back-lower = 背面

## The Edge profile panel

profile-assembly = アセンブリ：内部の部品をプロファイル加工します。
profile-shape = 形状
profile-size = サイズ
profile-around-face = 面の周囲
profile-along-length = 長手方向
# A corner named by the two faces meeting at it: "Top-left".
profile-corner = { $first }－{ $second }
# Which face the edges are taken around: "Around the top".
profile-around = { $side }の周囲
# One edge of that face: "Left edge".
profile-edge = { $edge }エッジ
# $profile is the router bit's name (Roundover, Chamfer), which is not translated yet.
profile-rout = { $profile }をルーター加工
profile-routed = { $size }の{ $profile }を加工しました。
profile-edge-profiles = エッジプロファイル

## Entity Info's part section: size, stock, grain, reference marks, cut and movement

part-assembly = アセンブリ：内部のグループとコンポーネントが部品です。
part-size = サイズ
part-size-hint = 長さ（木目方向）×幅×厚さ
part-stock = 材料寸法
part-needs-material = 材料が必要
part-stock-line = { $material }、{ $stock }
part-stock-glued-up = { $material }、{ $stock }、{ $boards }枚を接着
part-grain = 木目
part-grain-along-side = { $length }側に沿う
part-grain-along-length = 長さ方向に沿う
part-grain-along-length-hint = 最も長い辺（通常の場合）
part-reference = 基準
part-reference-hint = 作業工程表で寸法の基準にする面・エッジ・端部（木工でいう基準面と基準エッジ）
part-reference-face-edge = 面{ $face }、エッジ{ $edge }
part-reference-end = 端部{ $end }
part-reference-not-set = 未設定
# The end is the one mark with no default, so saying it isn't set would be misleading.
part-reference-as-drawn = 描画時のまま
part-pick = 選択
part-pick-hint = 次に部品をクリックします。幅の広い面が基準面、長いエッジが基準エッジ、端部が基準端部になります
part-clear = クリア
part-cut = 木取り
part-cut-unknown-hint = 最も伸縮する、平割りとして伸縮をチェックします
part-cut-flat-hint = 平割り：木口で年輪が横方向に走り、最も伸縮します
part-cut-quarter-hint = 年輪が木口で縦方向に走り、伸縮は平割りの約半分です
part-movement = 伸縮
part-free-to-move = 動けるように固定
part-free-to-move-hint = 天板用ボタン、8の字金具、長穴のねじなどで、接着や完全なねじ止めをせず木目を横切って動ける状態です
part-glue-up = はぎ合わせ
part-board-one = 1枚の板材
part-boards-n = 板材 { $count }枚
part-glue-auto = 自動（{ $boards }）
part-glue-one = 1枚の板
part-glue-from = { $count }枚から
part-edge-joint = 端接ぎ
# $piece is a biscuit or a loose tenon, named by the joinery; English makes it plural with an s.
part-edge-joint-hint = 各継ぎ目に{ $piece }、約{ $spacing }ごと
part-edge-glued-hint = 端接ぎ：よく接合された接着線は木材より強くなります

## The Templates panel: a cabinet, drawer, face frame or table from a few sizes

template-wood = 木材
# $template is the template's own name (Cabinet, Drawer), not translated yet.
template-add = { $template }を追加
template-add-hint = 図面の横に1つのグループとして作成します。元に戻すを1回実行すると取り消せます
template-change-size = サイズを変更
# A component built from a recipe, whose numbers are its own (the Templates panel's cousin).
# $sum is the arithmetic a parameter is worked out by, and arrives as it was written.
component-worked-out = { $sum }、計算済み
component-copies = このような{ $count }個のうちの1つです。ここで変更するとこの部品だけ固有の設計になり、他はそのまま残ります。
component-changed-by-hand = 作成後に手動で変更されています。寸法から再作成すると、その変更は失われます。
component-change-anyway = それでもサイズを変更
component-make = 固有の寸法を設定
component-make-hint = 構成を読み取り、後で寸法をまとめて名前付け・変更できるようにします。作成や移動は行いません。
# $notes is what the reader couldn't describe, in its own words.
component-made-notes = 一部を記述できないため、再作成しても同じ形にはなりません：{ $notes }
component-numbers = 構成
component-along = 長さ
component-across = 幅
component-thick = 厚さ
component-corner = 起点
component-size-hint = 各欄にはサイズ、または上の寸法による合計を入力します：幅 - 2*框
component-new-name = 名前
component-new-value = 値
component-add = 追加
template-change-size-hint = 現在の位置で、指定した寸法と木材を使って再作成します。内部で手動変更した内容は失われます。
template-width = 幅
template-height = 高さ
template-depth = 奥行き
template-stock = 材料寸法
template-top = 上
template-shelves = 棚板
template-adjustable = 調整可能（棚受けピン用）
template-back = 背面
template-back-ply = 1/4"合板
template-joints = 接合部
template-corners = コーナー
template-stiles = 縦框
template-rails = 横框
template-legs = 脚
template-aprons = 幕板
template-middle-rails = 中桟
template-fronts = 前板
template-pulls = 引き出し前板
template-doors = 扉
# A door on its own: a slab, or a frame around a floating panel.
template-style = 形式
template-panel = パネル
template-hinge = 片開き扉のちょうつがい
template-drawer-corners = 引き出しのコーナー
# The openings in a cabinet, named top-down as you look at it. A cabinet with no shelves has
# just the one, so it needs no number.
template-opening = 開口部
template-opening-bottom = 下側の開口
template-opening-top = 上側の開口
template-opening-n = 開口 { $n }
template-note-cabinet = 材料の側板・天板・底板・棚板。外寸です。引き出しは側付けスライドで動き、24"を超える扉は2枚に分かれます。
template-note-door = 縦框は全高、横框はその間に配置し、合板パネルを四周の溝に浮かせます。
    開口部の周囲の隙間分は差し引かず、扉そのものの寸法です。
template-note-drawer = 1/4"合板の底板を溝に入れた箱です。外寸です。
template-note-face-frame = 縦框は全高、横框はその間に配置します。
template-note-table = 角脚、厚さ3/4"の幕板、1"張り出した天板。高さは天板上面までです。

## The Joinery panel: the joint between two selected parts
##
## $part and $into are the parts' own names, as the drawing calls them, and $kind and $detail
## are the joinery's own words. They arrive as they are.

joinery-kind-let-in = 大入れ・溝・しゃくり
joinery-kind-overlap = 重なり部分を切削
joinery-kind-mortise-tenon = ほぞ穴とほぞ
joinery-kind-dowels = ダボ
joinery-kind-biscuits = ビスケット
joinery-kind-loose-tenons = ルーズテノン（ドミノ式）
joinery-kind-pocket-screws = ポケットねじ
joinery-kind-dovetails = あり組み
joinery-kind-splines = スプライン
joinery-kind-box-joint = あられ組み
joinery-kind-half-lap = 相欠き

# How the two parts have to be sitting before each joint can be made.
joinery-how-let-in = 一方の部品の端やエッジをもう一方に突き合わせます（側板に棚板、柱にパネルなど）。深さを設けて入れます。木目を横切る大入れ、木目に沿う溝、端部のしゃくりが該当します。
joinery-how-mortise-tenon = 横框の端をもう一方の部品に突き合わせます（脚や縦框に横框など）。
joinery-how-butt = 一方の部品の端やエッジをもう一方に突き合わせます。
joinery-how-dovetails = 一方の板の端を、もう一方の端の内側に突き合わせます。引き出しの前板に側板を接するような形です。突き合わせる板にテールを作ります。
joinery-how-box-joint = 2つの部品の端をコーナーで合わせます（引き出しの前板と側板など）。突き合わせても重ねても構いません。
joinery-how-half-lap = 2枚の板を交差させ、厚さ全体で重なるようにします。
joinery-how-overlap = 接合部の深さ分だけ一方の部品をもう一方へ移動します（側板に棚板を1/4"入れるなど）。

joinery-elsewhere = これらの部品の別のコピーは、別の場所で接する箇所に接合されています（{ $joints }）。ここにはまだありません。
joinery-already = 接合済み：
joinery-change = 変更
joinery-change-hint = 取り外し、寸法をもとにもう一度設定します
joinery-remove = 削除
joinery-remove-hint = この接合を両方の部品から取り外します
joinery-removed = 接合を削除しました。
joinery-set-up-again = もう一度設定してから作成します。
joinery-add-another = 別の接合を追加
# What an unnamed part is called while describing a joint.
joinery-part-default = 部品
joinery-joint = 接合
joinery-place-first = { $joint }（先に部品を配置してください）
joinery-joints = 接合部
joinery-at = （{ $at }）
joinery-row = { $other }{ $at }との{ $kind }、{ $detail }

joinery-cut-into = { $into }に{ $cut }を切削
joinery-cut-into-hint = { $cut }のサイズは維持し、{ $into }に切削します
joinery-cut-in = { $into }に{ $kind }を切削しました。
joinery-make-half-lap = 相欠きを作成
joinery-half-lap-made = 相欠きを作成しました。
joinery-finger-width = フィンガー幅
joinery-finger-width-hint = フィンガー数が奇数になるよう丸めます
joinery-make-box-joint = あられ組みを作成
joinery-box-joint-made = あられ組みを作成しました。
joinery-end-meets = { $part }の端が{ $into }に接します。

joinery-depth = 深さ
joinery-depth-hint = どれだけ入れるか。この分だけ長くなります
joinery-let-into = { $part }を{ $into }に入れる
joinery-let-in-done = { $into }に{ $kind }を切削しました。{ $part }は{ $length }長くなっています。

joinery-through = 貫通
joinery-through-hint = テールがコーナーの両面に現れます
joinery-half-blind = 半盲
joinery-half-blind-hint = 引き出し前板のように、正面からは見えません
joinery-tails = テール
# The three dovetail hints say what the word means first: plenty of people meet these names
# for the first time in this panel.
joinery-tails-hint = 切削するテールの数。テールは板に作る扇形の部分で、引き出しの側板に沿って見える箇所です。数を増やして細くすると手加工らしい見た目になります。
joinery-pins = ピン
joinery-pins-hint = ピンは、もう一方の板でテールの間の隙間を埋めるくさび状の部分です。見える部分の幅を指定します。細い（1/8"～1/4"）ものは手加工、テールと同じ幅のものは機械加工向きです。
joinery-half-pins = ハーフピン
joinery-half-pins-hint = ハーフピンは上下の端にあり、片側だけにテールがあります。コーナーを保持するため、通常はテール間のピンより少し幅広くします。
joinery-dovetail-note = { $part }にテール、{ $into }にピンを作成します（勾配1:8）。
joinery-cut-dovetails = あり組みを切削
joinery-dovetails-done = あり組みを切削しました。テールの分だけ{ $part }が長くなっています。

joinery-spline-note = 接合部全体に{ $piece }を入れ、各部品の溝に接着します。
joinery-cut-spline = スプライン溝を切削
joinery-spline-done = 両方の部品にスプライン溝を切削しました。

joinery-size = サイズ
joinery-count = 数
joinery-count-hint-slots = 接合部全体に均等に配置
joinery-fit = クリアランス
joinery-fit-hint-slots = 部材より溝をどれだけ大きくするか（厚く、長く、深く）。きつい嵌合は0
# The two mortise depths, for a machine that can only be set to certain ones. $depth is the
# depth WoodSketch would cut, in millimeters for a loose tenon and in the drawing's units
# otherwise; it arrives ready to read.
joinery-face-depth = 面の深さ
joinery-face-depth-hint = 面にどれだけ深く溝を入れるか。{ $depth }にする場合は空欄のままにするか、機械で設定できる値を入力します
joinery-end-depth = 端部の深さ
joinery-end-depth-hint = 差し込む部品の端部にどれだけ深く入れるか。{ $depth }にする場合は空欄のままにします
joinery-cut-biscuit-slots = ビスケット溝を切削
joinery-cut-loose-tenon-mortises = ルーズテノンのほぞ穴を切削
joinery-slots-done = 各部品に{ $count }個の溝を切削しました（{ $count } × { $piece }）。

joinery-count-hint-pocket = 約4"ごとに1本
joinery-screw = ねじ
joinery-screw-hint = 貫通する部品の厚さに合わせた長さ
joinery-pocket-note = { $part }の見えない面から穴を開けます。穴は描画しません。
joinery-add-pocket-screws = ポケットねじを追加
joinery-pocket-done = ポケットねじを{ $count }本追加しました。

joinery-through-tenon = 貫通ほぞ（長さ{ $length }、反対側に現れます）
joinery-haunch = ほぞ肩（端部のパネル溝を埋めます）
joinery-length = 長さ
joinery-length-hint = ほぞをほぞ穴へどれだけ入れるか
joinery-thickness = 厚さ
joinery-thickness-hint = 通常は横框の約3分の1
joinery-shoulders = ほぞ肩
joinery-shoulders-hint = 横框の両側のエッジから内側に設定
joinery-fit-hint-mortise = ほぞ穴をほぞよりどれだけ大きくするか（幅・厚さ・深さ）。きつい嵌合は0
joinery-tenon-mortise = { $part }にほぞ、{ $into }にほぞ穴
joinery-mortise-tenon-done = ほぞとほぞ穴を作成しました。木取り表では{ $part }が{ $length }長くなっています。

joinery-count-hint-dowels = 幅全体に均等に配置
joinery-diameter = 直径
joinery-diameter-hint = 3/4"材では通常3/8"
joinery-depth-hint-dowels = 各部品の穴の深さ
joinery-fit-hint-dowels = ダボより穴をどれだけ大きくするか。きつい嵌合は0
joinery-drill-dowels = ダボ穴を開ける
joinery-dowels-done = 各部品にダボ穴を{ $count }個開けました。

## Angled joints, where two parts meet at something other than a right angle

joinery-meet-angle = { $angle }°で接します。
joinery-draw-into = 接合位置で一方の部品の端をもう一方へ入れます。重なり量で深さが決まります。
joinery-angled-mortise-tenon-hint = 中央を通るほぞを、同じ形のほぞ穴へ入れます
joinery-housing = 受け溝
joinery-housing-hint = 端部全体をもう一方の部品へ入れます
joinery-every-copy = すべてのコピーを切削
joinery-every-copy-hint = 共有コンポーネントのコピーです。すべて同じように切削するか、これだけを切削します（独自のコンポーネントになります）
joinery-let-into-done = { $part }を{ $into }に入れました。
joinery-angled-thickness-hint = ほぞを作る部品の中央を通します。通常は約3分の1です
joinery-angled-shoulders-hint = 両側のエッジから内側に設定
joinery-angled-fit-hint = ほぞをほぞ穴よりどれだけ小さくするか（薄く、狭く、短く）。きつい嵌合は0
joinery-angled-done = 角度付きのほぞとほぞ穴を作成しました。{ $part }の端を{ $into }の面に合わせて切り、ほぞを入れています。

## The Cut List window

cutlist-title = 木取り表
cutlist-tab-parts = 部品
cutlist-tab-diagrams = 切断図
cutlist-tab-movement = 木材の伸縮
cutlist-tab-settings = 切断幅
cutlist-print-cut-sheet = 木取り表を印刷…
cutlist-print-cut-sheet-hint = 部品と切断図を印刷用ページとして保存します。ブラウザからPDFにもできます
cutlist-print-build-sheets = 作業工程表を印刷…
cutlist-print-build-sheets-hint = 作業用に、部品ごとのサイズ、図面、接合、加工を順番にまとめたページ
cutlist-export-csv = CSVを書き出し…
cutlist-save-failed = { $path }を保存できませんでした：{ $error }
cutlist-saved-not-opened = 保存しましたが開けませんでした：{ $error }
cutlist-kerf-note = 各切断で失われる切断幅。図面と一緒に保存されます
cutlist-stock-note = 材料の厚さ、シート寸法、製材の断面と長さは材料パネルで設定します
cutlist-saw-kerf = 切断幅
cutlist-kerf-unreadable = 切断幅を長さとして解釈できません。最後に有効だった値を使います

cutlist-no-parts = 切断する部品はまだありません。
# How many parts still have no material. The verb agrees with the number in English, which is
# why this is one message and not a sentence glued together from three.
cutlist-needs-material =
    { $count ->
        [one] { $warning } { $count }個の部品に材料が必要です（先頭に表示）。各部品、またはそれらを囲むグループを塗装してください。
       *[other] { $warning } { $count }個の部品に材料が必要です（先頭に表示）。各部品、またはそれらを囲むグループを塗装してください。
    }
cutlist-loose-faces = { $count }個の面がグループに含まれていないため、部品として扱われません。木取り表に載せるにはグループ化してください
cutlist-needs-a-material = 材料が必要です
cutlist-parts-count =
    { $count ->
        [one] { $count }個の部品
       *[other] { $count }個の部品
    }
cutlist-material-line = { $kind }  ·  { $parts }  ·  { $amount }
cutlist-edit-material = 材料を編集
cutlist-square-feet = { $amount }平方フィート
cutlist-linear-feet = { $amount }延べフィート
cutlist-board-feet = { $amount }ボードフィート
cutlist-pieces = { $count }個

# The parts table's column headings.
cutlist-column-name = 名前
cutlist-column-qty = 数量
cutlist-column-length = 長さ
cutlist-column-width = 幅
cutlist-column-thickness = 厚さ
cutlist-column-stock = 材料寸法
cutlist-glued-up = { $stock }、{ $boards }枚をはぎ合わせ
cutlist-glued-up-hint = { $boards }枚を約{ $width } x { $length }で木端接ぎし、接着してからサイズに切断
cutlist-no-stock = なし
cutlist-hardware = 金物

cutlist-nothing-to-cut = 切断する部品はまだありません
cutlist-hardware-not-cut = 金物は材料寸法から切り出しません
cutlist-paint-to-plan = 部品に材料を塗り、その種類を設定して木取りを計画します
cutlist-type-not-cut = この材料種類は材料寸法から切り出しません。シート材、無垢材、または規格製材に設定してください
cutlist-buy = 購入：
cutlist-buy-line = { $count } × { $what }
cutlist-used = （{ $percent }%使用）
cutlist-part-problem = 部品{ $label }：{ $reason }
cutlist-stock-line = { $n }. { $description }  ·  使用率{ $percent }%
cutlist-not-from-stock = 材料寸法から切り出しません（材料種類：その他）：{ $materials }

## The Wood movement tab

movement-intro = 無垢材は季節の変化に伴い、木目を横切る方向に膨張・収縮します。合板とMDFはそのように動きません。
movement-swing = 含水率の変動
movement-swing-hint = 1年間で木材の含水率がどれだけ変化するか。屋内では約4%（乾燥した冬の6%から湿った夏の10%）。物置や暖房のない部屋ではさらに大きくなります
movement-unknown = { $info } { $materials }の伸縮データがありません。部品を確認するには、材料パネルで「伸縮特性」を設定してください。
movement-none-known = 樹種が判明している無垢材部品はまだありません。ライブラリのチェリーやレッドオークなどを部品に塗装してください。
movement-no-conflicts = { $check } 互いの伸縮を妨げる部品はありません。
movement-watch = この接合に注意
movement-fastened-note = 部品を動けるように固定している場合（天板用ボタン、8の字金具、長穴のねじ穴など）は、エンティティ情報で「動けるように固定」を選択してください。
movement-allowed = 許容範囲
movement-how-much = 各部品が木目を横切る方向に動く量
movement-column-part = 部品
movement-column-wood = 木材
movement-column-width = 幅
movement-column-moves = 伸縮量
movement-assumed-flat-hint = 木取りが「不明」なので、最も伸縮する平割りとして計算しています。エンティティ情報で木取りを設定してください。

## The Start page

start-back-to = ‹  { $drawing }に戻る
start-back-to-hint = Esc
start-title = スタート
start-intro = 図面を作成するか、作業中の図面を開くか、サンプルを分解してみましょう。
start-new-drawing = 新しい図面
start-inches = インチ
start-millimeters = ミリメートル
start-open = 開く…
start-open-under = 図面またはSketchUpモデル（Ctrl+O）
start-import = 読み込む…
start-import-under = 開いている図面にSketchUpモデルを読み込みます
start-recent = 最近使った項目
start-recent-empty = 開いたり保存した図面がここに表示されます。
start-new = 新規
start-samples = サンプル
start-samples-note = 開くたびに新しく作成される完成品です。自由に変更できます。ファイルではないため、ここに上書き保存されることはありません。
# How a card reads out to a screen reader: its title, then the line under it.
start-card-label = { $title }. { $under }

## Opening, saving and the title bar

files-untitled = 無題
files-save-failed = { $path }を保存できませんでした：{ $error }
files-open-failed = { $path }を開けませんでした：{ $error }

# The graphics card reported a problem. The drawing is safe and can be saved; what is on screen
# may not have drawn. Help > Report a Problem carries the card's own words.
gpu-trouble = グラフィックスカードから問題が報告されました：{ $error }。図面は安全です。保存してから、ヘルプ > 問題を報告 で報告してください。
# WoodSketch is drawing with a different graphics backend because the one it prefers went wrong
# on this computer. $backend is the one in use now, $failed the one given up on (Vulkan,
# DirectX 12, OpenGL, Metal: the names their makers use, not translated).
status-graphics-fell-back = { $backend }で描画中 — このコンピューターでは{ $failed }が動作しませんでした
status-graphics-fell-back-hint = ここでは{ $failed }が失敗したため、WoodSketchは自動的に別の方式へ切り替えました。新しいバージョンでは再試行します。
files-export-failed = { $path }を書き出せませんでした：{ $error }
# The window's title: the drawing's name, a star if there are unsaved changes, then the app.
files-window-title = { $name }{ $dirty } - WoodSketch
# Dismisses a message about a file that couldn't be opened or saved.
files-ok = OK
files-save-changes = 変更を保存しますか？
files-save-changes-question = { $name }への変更を保存しますか？
files-save = 保存
files-dont-save = 保存しない
files-cancel = キャンセル
# The named kinds of file in the system Open and Save dialogs.
files-filter-drawing = WoodSketch図面
files-filter-drawings-and-skp = 図面とSketchUpモデル
files-filter-sketchup = SketchUpモデル（2021以降）
files-filter-web-page = Webページ

## Report a Problem and Send Feedback

feedback-problem-title = 問題を報告
feedback-idea-title = フィードバックを送信
feedback-unreachable = woodsketch.netに接続できませんでした（{ $error }）。接続を確認するか、GitHubで開いてください。
feedback-after-crash = WoodSketchが予期せず終了しました。行っていた操作：
feedback-thanks = { $check } ありがとうございます。GitHubのissue #{ $number }です：
feedback-follow-email = そこで返信を確認してください。質問がある場合はメールが届きます。
feedback-follow = そこで返信を確認してください（読むだけならアカウントは不要です）。
feedback-close = 閉じる
feedback-kind-problem = 問題が発生しました
feedback-kind-idea = アイデアまたはその他のフィードバック
feedback-title-label = タイトル
feedback-what-happened = 何が起きましたか？（行った操作と期待した結果）
feedback-your-idea = アイデア
feedback-email = メールアドレス（任意）
feedback-email-hint = 追加の質問を受けてもよい場合のみ
feedback-email-note = この報告について質問するため、開発者だけが確認します。公開はされません。
feedback-include-counts = 開いている図面の件数を含める（名前は含めません）
feedback-include-counts-hint = 種類ごとの部品、面、接合、材料の数。問題の再現に役立ちます
feedback-sent-with = 送信内容：
feedback-public-note = GitHubで公開issueになり、そこで進捗を確認できます。送信にアカウントは不要です。
feedback-bad-email = メールアドレスの形式が正しくないようです。
feedback-send = 送信
feedback-sending = 送信中…
feedback-open-github = GitHubで開く
feedback-open-github-hint = GitHubで自分で送信します（無料アカウントが必要です）。メールアドレスは含まれません
feedback-cancel = キャンセル

## Checking for updates (Help > Check for Updates)

updates-checking = 更新を確認中…
updates-not-installed = このコピーはWoodSketchインストーラーでインストールされていないため、自動更新できません。woodsketch.netから最新版をダウンロードしてください。
updates-up-to-date = WoodSketch { $version }は最新です。
updates-downloading = WoodSketch { $version }をバックグラウンドでダウンロード中です（右下に表示）。準備ができると、そこに緑色の「再起動して更新」ボタンが表示されます。
updates-ready = WoodSketch { $version }の準備ができました。再起動して更新してください。
updates-failed = 更新を確認できませんでした：{ $error }

## While a sample is being built

building-title = 作成中
building-what = { $sample }を作成中
building-note = 現在のアプリにある接合方法を使い、毎回レシピから作成します。

## Opening and importing, crash reports, recovery, and the file-changed question

app-opened-skipped = 開きましたが、{ $count }個の面を読み取れなかったため除外しました。
app-imported-skipped = 読み込みましたが、{ $count }個の面を読み取れなかったため除外しました。
app-skp-empty = そのSketchUpモデルは空です。
app-crash-title = WoodSketchで問題が発生しました
app-crash-note = 前回、予期せず終了しました（{ $version }）。報告を送ると、WoodSketchのコードのどこで何が起きたかを確認して修正に役立てられます。送信前に内容をすべて確認できます。
# When the last run stopped leaving nothing behind at all, so there is no fault to name and
# nowhere in the code to point at. Said differently from the note above, which promises both.
app-crash-note-quiet = 前回（{ $version }）、警告なく停止し、理由を示す情報が何も残りませんでした。通常はグラフィックスドライバーが原因です。使用中のグラフィックスカードとドライバーを含む報告は役立ちます。送信前に内容をすべて確認できます。
app-crash-report = 報告する…
app-crash-not-now = 後で
# What a drawing with no file of its own is called in the question below.
app-the-drawing = 図面
app-file-changed-title = WoodSketchの外でファイルが変更されました
app-file-changed = { $name }が別の操作で変更されました。この図面にも未保存の変更があります。保持できるのはどちらか一方だけです。
app-reload = ファイルから再読み込み
app-reload-hint = 現在のファイルを読み込み、ここでの未保存の変更を失います（Ctrl+Zで戻せます）
app-keep-mine = 自分の変更を保持
app-keep-mine-hint = この図面をそのままにします。保存するとファイルを上書きします
app-recover-title = 未保存の作業を復元しますか？
app-recover-note = 前回、WoodSketchは正常に終了しませんでした。未保存の変更のコピーを保持しています：
app-recover-minutes = 、{ $minutes }分前
app-recover-hours = 、{ $hours }時間前
app-recover-line = { $name }{ $when }
app-recover = 復元
app-discard = 破棄
app-discard-all = すべて破棄
app-build-sheets-title = { $name }の作業工程表
app-autosave-failed = 自動保存できませんでした：{ $error }
# The headings on the printable pages, which open in a browser.
cutlist-sheet-title = 木取り表
cutlist-build-sheets-title = 作業工程表

## The status bar's left-hand side: what the tool in your hand is waiting for.
##
## These are read while working, so they are terse on purpose, and they name keys (Ctrl, Esc,
## W A S D) that are the same on every keyboard. Keep the key, translate around it.

status-pick-reference = 部品の基準面（幅の広い面）、エッジ（長い辺）、または端部をクリックします。Escで停止します。
status-select = オブジェクトを選択します。Ctrlで追加、Shiftで切替、Ctrl+Shiftで削除します。右へドラッグすると包含選択、左へドラッグすると交差選択になります。
status-orbit = ドラッグして回転します。
status-pan = ドラッグして移動します。
status-zoom = 上へドラッグすると拡大、下へドラッグすると縮小します。
status-walk = ウォーク中：マウスを動かして視線を変えます。W A S Dまたは矢印キーで歩きます（Shiftで高速）。EとQで上下、Fで飛行し、ホイールで速度を設定します。目の高さを入力できます。壁は止めませんが、床が支えます。Escで停止します。

# What the Measurements box at the bottom right is asking for.
measure-dimensions = 寸法
measure-angle = 角度
measure-length = 長さ
measure-radius = 半径
measure-bulge = 膨らみ
measure-scale = 倍率
measure-distance = 距離
measure-text = テキスト
measure-eye-height = 目の高さ
measure-measurements = 測定値

# Inference: which axis the cursor has lined up with. The colors are the drawing's own -- red is
# X, green is Y, blue is Z, as in SketchUp -- so they are names here, not just colors.
infer-on-red = 赤軸上
infer-on-green = 緑軸上
infer-on-blue = 青軸上
infer-parallel = エッジに平行
infer-perpendicular = エッジに垂直

tools-part-gone = その部品はありません。
tools-click-part = 部品そのものをクリックしてください。
tools-click-part-reference = 部品そのものをクリックして、基準面または基準エッジを選択してください。
tools-opened-group = { $name }を開いて編集しています。Escまたは外側のクリックで閉じます。
tools-select-to-group = { $noun }にするものを選択してください。
tools-noun-group = グループ
tools-noun-component = コンポーネント
tools-no-guides = 削除できるガイドはありません。
tools-select-to-explode = 分解するグループまたはコンポーネントを選択してください。
tools-select-to-flip = 反転するものを選択してください。
tools-select-to-copy = コピーするものを選択してください
tools-nothing-to-paste = 貼り付けるものがありません。先にコピーしてください

## What each tool says while you use it, and what it says when it can't do what was asked.
##
## `[x,y,z]`, `3x`, `/3`, `24s` and `5r` are typed into the Measurements box and are the same in
## every language: keep them exactly, and translate the words around them.

# Snapping: what the point under the cursor is.
snap-endpoint = 端点
snap-origin = 原点
snap-guide-point = ガイド点
snap-intersection = 交点
snap-midpoint = 中点
snap-on-edge = エッジ上
snap-on-guide = ガイド上
snap-on-face = 面上

# Arc (two points and a bulge)
tool-arc-start = 始点を選択（または[x,y,z]を入力）。辺数：{ $sides }（変更するには24sと入力）。
tool-arc-end = 終点を選択するか、弦の長さを入力します。
tool-arc-bulge = 膨らみを選択するか入力します（半径は5r）。
tool-arc-click-start-end = 円弧の始点、次に終点をクリック
tool-arc-radius-min = 半径は弦の半分以上でなければなりません（{ $half }）
tool-arc-end-away = 始点から離れた終点を選択

# Center arc, and the three-point arc
tool-arc-center = 中心を選択。辺数：{ $sides }（変更するには24sと入力）。矢印キーで軸に沿わせます。
tool-arc-where-starts = 円弧の始点を選択するか、半径を入力します。
tool-arc-turn-and-click = 回転して終点をクリックするか、角度を入力します。
tool-arc-start-away = 中心から離れた始点を選択
tool-arc-click-center-first = 先に中心をクリック
tool-arc-turn-away = 始点から離れる方向へ回転して円弧を描きます
tool-arc-three-first = 円弧の始点を選択。辺数：{ $sides }（変更するには24sと入力）。
tool-arc-three-second = 円弧が通る点を選択。
tool-arc-three-third = 円弧の終点を選択。
tool-arc-second-away = 1点目から離れた2点目を選択
tool-arc-in-a-line = 3点が一直線上にあります。線上にない終点を選択してください
tool-arc-three-points = 3点（始点、円弧上の点、終点）をクリック。24辺なら24sと入力します。

# Circle and Polygon
tool-circle-sides = 辺数は3～999の整数で指定します（例：24s）
tool-circle-center-then-radius = 中心をクリックし、半径を入力
tool-circle-corner = 円の角を選択するか、半径を入力します。辺数：{ $sides }（変更するには8sと入力）。
tool-polygon-corner = 多角形の角を選択するか、半径を入力します。辺数：{ $sides }（変更するには8sと入力）。
tool-circle-center = 中心点を選択（または[x,y,z]を入力）。辺数：{ $sides }（変更するには32sと入力）。矢印キーで軸に沿わせます。
tool-polygon-center-to-corner = 中心点を選択（または[x,y,z]を入力）。辺数：{ $sides }（変更するには8sと入力）。矢印キーで軸に沿わせます。角までの半径（Ctrlを押すと辺まで）。
tool-polygon-center-to-side = 中心点を選択（または[x,y,z]を入力）。辺数：{ $sides }（変更するには8sと入力）。矢印キーで軸に沿わせます。辺の中央までの半径（Ctrlを押すと角まで）。
tool-circle-through-point = 通過点
tool-circle-move-away = 中心から離れる方向へ移動するか、半径を入力
tool-circle-radius-positive = 半径は0より大きくしてください

# Eraser, Line, Offset, Paint Bucket
tool-eraser-status = エッジ、ガイド、グループをクリックまたはドラッグして消去します。
tool-eraser-click = エッジ、ガイド、グループ、コンポーネントをクリックして消去
tool-line-start = 始点を選択するか、[x,y,z]を入力。
tool-line-end = 終点を選択するか値を入力。Shiftを押したまま、または矢印キーを押して方向を固定。Escで線を終了します。
tool-line-length-positive = 長さは0より大きくしてください
tool-line-click-start = 始点をクリック（または[x,y,z]を入力）し、長さを入力
tool-offset-pick = 面をクリックして輪郭をオフセット（または先に面を選択）。
tool-offset-move = 内外へ移動してクリックするか、オフセット距離を入力します。
tool-offset-click-face = 面をクリックして輪郭をオフセット
tool-offset-click-then-type = 面をクリックし、オフセットを入力
tool-offset-move-first = 先に面の内側または外側へマウスを移動
tool-paint-status = 材料パネルで選択した材料を板全体に塗装するには、板をクリックします。Ctrl+クリック：1つの面。Alt+クリック：スポイト。
tool-paint-click = グループ、コンポーネント、または面をクリックして塗装します。

# Push/Pull
tool-pushpull-pick = 面を選択して押し出します。Ctrlで面を保持します。面をダブルクリックすると前回の距離を繰り返します。
tool-pushpull-keeping-pick = 面を保持（Ctrl）：面を選択すると、そこから新しい面と壁が作られます。
tool-pushpull-move = 移動して距離を決めてからクリックするか、値を入力します。Ctrlで面を保持します。
tool-pushpull-keeping = 面を保持：新しい面と壁が作られます。移動して距離を決めてからクリックするか、値を入力します。
tool-pushpull-click-then-type = 面をクリックし、距離を入力
tool-pushpull-standard-thickness = 標準の厚さ
tool-pushpull-in-group = その面はグループの一部です。グループをダブルクリックして内部を編集し、押し出してください。

# Rectangle, and the rotated rectangle
tool-rect-first = 1つ目の角を選択するか、[x,y,z]を入力。
tool-rect-opposite = 対角の角を選択するか、値を入力。
tool-rect-square = 正方形
tool-rect-golden = 黄金比
tool-rect-enter-wh = 幅と高さを入力（例：1.5",3.5"）
tool-rect-dims-positive = 寸法は0より大きくしてください
tool-rect-click-first = 1つ目の角をクリックし、サイズを入力
tool-rotrect-first = 1つ目の角を選択。
tool-rotrect-side = 2つ目の角（1辺目）を選択するか、その長さを入力。
tool-rotrect-across = 横方向へ移動してクリックするか、もう一方の辺の長さを入力。
tool-rotrect-second-away = 1つ目の角から離れた2つ目の角を選択
tool-rotrect-click-first = 1つ目の角をクリック
tool-rotrect-move-across = 1辺目を横切るように移動して長方形の幅を決めます

# Scale
tool-scale-pick-grip = ハンドルをクリック：角は均等に、エッジは2方向に、面は1方向に拡大縮小します。
tool-scale-move-center = 移動して拡大縮小するか、倍率（2、0.5）またはサイズ（36"）を入力します。中心を基準にします（Ctrlで反対側を固定）。Shiftで均等と伸縮を切り替えます。
tool-scale-move-opposite = 移動して拡大縮小するか、倍率（2、0.5）またはサイズ（36"）を入力します。Ctrlで中心を基準にします。Shiftで均等と伸縮を切り替えます。
tool-scale-click-grip = 拡大縮小ハンドルをクリックし、倍率を入力
tool-scale-hover-grips = 拡大縮小ハンドルの1つをクリック（何かにカーソルを合わせると表示）
tool-scale-not-negative = 拡大縮小率は0より大きくしてください。反転するには編集 > 反転方向を使います。
# $value is what was typed, kept as it was typed.
tool-scale-not-a-factor = 「{ $value }」は倍率ではありません
tool-scale-one-or-n = 1つの値、またはカンマ区切りで{ $count }個の値を入力

# Ctrl+D with nothing picked.
tool-nothing-to-duplicate = 複製するものを選択

# Tape Measure
tape-note-guide-line = + ガイド線
tape-note-guide-point = + ガイド点
tape-note-measure = 測定
tool-tape-guide-lines = ガイド線を作成：エッジをクリックしてそこからガイドをオフセットするか、点をクリックします。Ctrlでモードを変更。
tool-tape-guide-points = ガイド点を作成：測定の基準点をクリックします。Ctrlでモードを変更。
tool-tape-measure-mode = 測定：基準点をクリックするか、エッジにカーソルを合わせます。測定後に長さを入力してモデルのサイズを変更します。Ctrlでモードを変更。
tool-tape-move-away = エッジから離れ、クリックするかガイドのオフセットを入力します。
tool-tape-click-to = 測定先の点をクリック。
tool-tape-click-to-or-type = 測定先の点をクリックするか、距離を入力。
tool-tape-show-direction = マウスを動かして方向を示し、距離を入力
tool-tape-measure-first = 距離を測定し、設定する長さを入力
tool-tape-distance-positive = 距離は0より大きくしてください
tool-tape-click-from = 基準点をクリックし、距離を入力

# Dimensions and text notes
tool-dim-first = 1つ目の測定点を選択するか、エッジをクリックして寸法を付けます。
tool-dim-second = 2つ目の点を選択。
tool-dim-place = 外側へ移動して寸法の位置を決めてからクリックするか、離す距離を入力します。
tool-dim-second-away = 1つ目から離れた2つ目の点を選択
tool-dim-two-points = 2点を選択し、寸法をどれだけ外側へ出すか入力
tool-note-pick-point = 注記の対象となる点を選択。
tool-note-where-words = 文字を配置する場所をクリック（同じ位置なので引出線はありません）。
tool-note-type-words = 注記の文字を入力してEnterを押すか、別の注記の点を選択します。
tool-note-place-first = 注記を配置し（点をクリックしてから文字の位置をクリック）、文字を入力

# Move. The copying ones are whole sentences rather than a sentence with "(copying)" dropped
# into it, because where that note belongs is a question about the language.
tool-move-base = 移動の基点を選択（または[x,y,z]を入力）。Ctrlでコピー。コピー後に3xまたは/3と入力すると配列になります。
tool-move-base-copying = 移動する基点を選択（コピー中）（または[x,y,z]を入力）。Ctrlでコピー。コピー後に3xまたは/3と入力すると配列になります。
tool-move-destination = 移動先を選択するか距離を入力。矢印キーで軸を固定し、Ctrlでコピーします。
tool-move-destination-copying = 移動先を選択するか距離を入力（コピー中）。矢印キーで軸を固定し、Ctrlでコピーします。
tool-move-pasting = クリックして配置（点にスナップ）するか、距離を入力。Escで取り消します。
tool-move-copy-first = 先にコピーし（移動中にCtrl）、3xまたは/3と入力
tool-move-arrays-need-copy = 配列にはコピーが必要です。移動中にCtrlを押してください
tool-move-click-base = 基点をクリックし、距離を入力
tool-move-select = 移動するものを選択するか、その上から開始
tool-move-away-from-base = 先に基点からマウスを離す
tool-move-vector = ベクトルをx,y,zで入力
tool-move-show-direction = マウスを動かして方向を示し、距離を入力

# Rotate, and the Protractor
tool-rotate-center = 回転の中心を選択。矢印キーで分度器を固定（右は赤、左は緑、上は青）。Ctrlでコピーします。
tool-rotate-center-copying = 回転の中心を選択（コピー中）。矢印キーで分度器を固定（右は赤、左は緑、上は青）。Ctrlでコピーします。
tool-rotate-start = 回転の始点を選択。
tool-rotate-start-copying = 回転の始点を選択（コピー中）。
tool-rotate-end = 回転の終点を選択するか、角度を入力。
tool-rotate-end-copying = 回転の終点を選択するか、角度を入力（コピー中）。
tool-protractor-center = 分度器の中心を選択（ガイド線が通ります）。矢印キーで固定（右は赤、左は緑、上は青）。
tool-protractor-start = 開始方向の点を選択。
tool-protractor-direction = ガイドの方向を選択するか、角度を入力。
tool-rotate-away-from-center = 中心から離れた点を選択
tool-rotate-copy-first = 先にコピーを回転し（回転中にCtrl）、3xまたは/3と入力
tool-rotate-arrays-need-copy = 配列にはコピーが必要です。回転中にCtrlを押してください
tool-rotate-pick-start-dir = 開始方向を選択し、角度を入力
tool-rotate-pick-center-dir = 中心と開始方向を選択し、角度を入力
tool-rotate-select = 回転するものを選択するか、その上から開始
tool-rotate-turn-away = 先に開始方向から離れるように回転
tool-rotate-invalid-angle = 無効な角度：「{ $typed }」

# The Board tool. $stock is the material and size picked in the Board panel ("Pine Lumber 2x4").
tool-board-no-material = 板材：材料パネルの＋で木材を追加し、その材料寸法から板材を作成してください。
tool-board-lumber-start = { $stock }：板材の始点をクリック。2x4x96、またはカスタムサイズなら3,6 1/2と入力します。Ctrlで板を立てます。
tool-board-sheet-start = { $stock }：板材の角をクリック。
tool-board-lumber-end = 終点をクリックするか長さを入力。矢印キーで軸を固定。Wで板材を反転します。
tool-board-sheet-end = 対角の角をクリックするか、長さ,幅を入力。Wで板材を反転します。
tool-board-add-material = 先に材料パネルの＋で木材を追加し、板材パネルで材料寸法を選択してください。
tool-board-length-first = 先に長さを決めます。始点から離れるように移動してください。
tool-board-no-size = その板材にはサイズがありません。
tool-board-pick-stock = 先に板材パネルで材料寸法を選択してください。
tool-board-click-start = 板材の始点をクリックし、長さを入力（または2x4x96と入力）。
tool-board-length-positive = 長さは0より大きくしてください。
tool-board-show-direction = 板材を伸ばす方向へマウスを移動し、長さを入力。
tool-board-length-width = 長さと幅を入力（例：30,11 1/4）
tool-board-no-thickness = そのサイズには厚さまたは幅がありません。
tool-board-pick-sheet = 先に板材パネルでシート材または無垢材を選択してください。

## The viewport: what is drawn over the drawing, and the right-click menu

viewport-editing = { $path }を編集中    Escまたは外側をクリックして閉じる
viewport-walk-speed = 歩行速度：1秒あたり{ $speed }
viewport-flying = 飛行中：支えるものはありません。EとQで上下し、Fで歩行に戻ります。
viewport-walking = 歩行中：床に戻りました。
viewport-grain = 木目
viewport-reference-face = 基準面
viewport-reference-edge = 基準エッジ
viewport-from-point = 基準点から
viewport-make-unique = 固有化
viewport-make-unique-hint = 固有のコピーなので、変更しても他のコピーには影響しません
viewport-reverse-faces = 面を反転
viewport-reverse-faces-hint = 選択した面を表裏反転
