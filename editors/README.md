# Editor support

```
editors/
  tree-sitter-roop/   the grammar: grammar.js, the generated src/, a test corpus
  zed/                the Zed extension: extension.toml and languages/roop/*.scm
  check.sh            checks all of it
```

The grammar follows `crates/roop-syntax/src/parser`. It is for highlighting,
outlines and structural editing, so it accepts a little more than the compiler,
and it reserves the words the compiler treats as contextual (`test`, `expect`,
`bennett`, `session`, `offer`, `select`, `checkpoint`, `rec`, `end`, `Stack`).
Run `editors/check.sh` after changing the language: it regenerates the parser,
runs the corpus, parses every `.roop` file of the repository and compiles the
queries against the grammar. `src/` is committed because Zed builds the
grammar from it. Change `grammar.js`, then `npm run generate` in
`tree-sitter-roop`, and commit both.

## Zed

Install the extension from a checkout with the command palette, `zed: install
dev extension`, and pick `editors/zed`. Zed fetches the grammar from the
repository and `rev` in `extension.toml`, so the commit must be pushed. To try
it before that, point `repository` at the checkout, as in
`repository = "file:///path/to/roop"`, and set `rev` to a commit that has the
grammar. `dev: open highlights tree view` shows what a theme colors.

`languages/roop` holds the queries Zed reads:

| File | Does |
| --- | --- |
| `highlights.scm` | colors, with Zed's capture names (`@keyword`, `@function`, `@variant`) |
| `brackets.scm` | matching and auto-closing of `()`, `[]`, `{}` |
| `indents.scm` | indentation after a block, argument list or body |
| `outline.scm` | the symbol outline: functions, tests, structs, enums, sessions, modules |

`tree-sitter-roop/queries/highlights.scm` is a link to the same file, so the
tree-sitter CLI and Zed use one copy.

## The language server, later

Highlighting needs no server. A server adds diagnostics, go to definition, hover
and semantic tokens, and most of what it needs is already in the compiler:

- `roop-syntax` parses with spans on every statement, block, item and error
  (`parse_spanned`, `comments`), which is also what `roop fmt` is built on.
- `roop-check`, `roop-modules`, `roop-opt` and `roop-lean` report errors with
  the span of what failed, which become diagnostics.
- `roop-fmt` formats a text, which is `textDocument/formatting`.
- `roop-modules` resolves `use` and `mod` through `Roop.toml`, which is go to
  definition across files.
- `roop test` and `roop lean` can report per function, which suits code lenses
  ("run test", "proved reversible").

The steps are a `roop-lsp` crate that speaks the protocol over stdio (and
`roop lsp` to start it), then a Zed extension with a small Rust part, built to
`wasm32-wasip2` with the `zed_extension_api` crate, that tells Zed to run that
command. The commented `[language_servers.roop-lsp]` block in
`zed/extension.toml` and the `language_servers` line in
`zed/languages/roop/config.toml` are where it plugs in. Until then Zed uses the
grammar alone.
