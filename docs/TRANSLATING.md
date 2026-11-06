# Translating WoodSketch

WoodSketch can speak any language someone is willing to write down, and adding one needs **no
Rust and no programming**. A translation is one text file. If you can use a text editor, you can
do this.

Everything the app says in English lives in one file:

```
crates/i18n/i18n/en-US.ftl
```

A translation is a copy of that file with the right-hand sides rewritten.

## What the file looks like

```ftl
menu-file = File
file-new = New
file-open = Open...
file-start-page-hint = New, Open, Recent and the samples; Esc comes back here
```

Left of the `=` is a **key**. It is the name the code knows the string by — **never translate a
key, and never change one.** Right of the `=` is the text people read, and that is the part you
rewrite:

```ftl
menu-file = Arquivo
file-new = Novo
file-open = Abrir...
file-start-page-hint = Novo, Abrir, Recentes e os exemplos; Esc volta para cá
```

A few things to know:

- **`-hint` is a tooltip** — the text that appears when the mouse rests on something. It can be a
  whole sentence, and often should be.
- **`{ $something }` is a value the app fills in.** Keep it exactly as it is, braces and all, but
  move it wherever the sentence needs it:
  ```ftl
  status-updated = Atualizado para { $version }
  ```
- **`...` at the end** of a menu item means "this will ask you more before doing anything". It is
  a Windows convention; keep it if your language has it, drop it if not.
- **Lines beginning with `#` are comments** for you, not text anybody sees. Several of them
  explain where a string appears — read them, they are there to save you guessing.
- **Punctuation belongs to you.** If your language spaces punctuation differently, or quotes
  differently, write it the way your language writes it.

The format is [Fluent](https://projectfluent.org/fluent/guide/), Mozilla's translation format. It
handles plurals and grammatical gender properly, so if a string needs a shape English has no
equivalent of, Fluent can say it — [the guide](https://projectfluent.org/fluent/guide/) covers
`{ $count ->` and friends when you get to one.

## Adding your language

1. **Copy `en-US.ftl`** to `crates/i18n/i18n/<your tag>.ftl`. The tag is a
   [BCP 47](https://www.w3.org/International/articles/language-tags/) code — `pt-BR` for
   Brazilian Portuguese, `de` for German, `fr-CA` for Canadian French. Include the region when
   the language differs between them, leave it off when it doesn't.

2. **Translate the right-hand sides.** You don't have to do all of them. Anything you leave in
   English, or delete, simply shows in English — the app falls back rather than showing a blank
   or a key. A half-finished translation is genuinely useful and genuinely shippable.

3. **Add one line** to `CATALOGS` in `crates/i18n/src/lib.rs`:

   ```rust
   Catalog { tag: "pt-BR", endonym: "Português (Brasil)", source: include_str!("../i18n/pt-BR.ftl") },
   ```

   `endonym` is your language's name **in your language** — `Português (Brasil)`, not
   `Portuguese (Brazil)`. Someone hunting for their own language in the menu can't necessarily
   read the one it would otherwise be listed under. This is the only line of Rust involved, and
   copying the line above and changing three strings is the whole job.

4. **Open a pull request.** If step 3 defeats you, open it with just the `.ftl` file and say so —
   adding the line takes ten seconds and is not worth you being blocked on.

That is all. The app reads the computer's language when it first runs and picks the closest match,
and **View > Language** lets anyone choose. There's no separate download per language: every
translation is built into the one program, which is why adding yours is a file and a line.

## Checking your work

If you have Rust installed (see [Build it yourself](../README.md#build-it-yourself)):

```sh
cargo test -p woodsketch-i18n
cargo run --release -p woodsketch
```

Two tests do the checking that matters:

- **`every_key_the_code_asks_for_exists`** reads the whole program for keys and looks each one up,
  so a typo in a key fails the build rather than appearing in somebody's menu.
- **`no_translation_says_what_english_no_longer_does`** tells you if your file has a key English
  has since dropped — dead weight to delete.

If you don't have Rust, open the pull request anyway and CI will run both for you.

## What isn't translated, and why

- **The command line tool and the MCP server** (`woodsketch-cli`) stay in English. They are read
  by programs and by people working with AI assistants, and English is the lingua franca there.
- **Material and species names** — Cherry, Baltic Birch Ply, 2x4 — are not translated. Some are
  closer to product names than words, and getting them right needs someone who buys timber in
  that language, not just someone who speaks it. Where one appears inside a sentence you are
  translating it arrives as `{ $material }` or similar, so you can put it wherever your language
  needs it.
- **Template names** (Cabinet, Drawer) are not translated, for the same reason as materials:
  they name a thing the drawing stores, and arrive inside sentences as `{ $template }`.

  **Tool names are** translated -- `tool-name-push-pull` and the rest. They are deliberately
  SketchUp's names, so if SketchUp has a translation in your language, use the name it uses:
  that is what someone switching over will be looking for. Keeping the English is a fair
  choice too, where that is what people actually say.
- **Typed measurements.** `3/4"`, `2'6`, `38mm` are parsed, not displayed, and don't go through
  translation. A comma is read as a decimal point, so `38,5` is thirty-eight and a half. Where
  several lengths are typed at once, separate them with a semicolon and the commas stay
  decimals: `5,5';6'`. A comma on its own still separates (`3,5` is two lengths), because it
  has meant that since the first version. Lengths are *shown* with a period, which is what the
  translator who asked for this preferred; if your language would rather see a comma there,
  say so.

Everything else a woodworker reads is in the catalog, including the line at the bottom of the
window that tells you what the tool in your hand is waiting for. If you find something that isn't,
that's a bug worth reporting — say where you saw it.
