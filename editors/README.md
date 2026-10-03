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

## The language server

`roop lsp` speaks the Language Server Protocol over stdin and stdout. It does
four things, for the files an editor has open:

- **Diagnostics.** A syntax error is reported where the parser stopped. When the
  file stands alone, with no `use` or `mod`, what the checker rejects is reported
  too, at the span the checker gives. A file that uses other modules gets only
  syntax errors, since the checker's spans would be in a program assembled from
  several files.
- **Formatting** (`textDocument/formatting`), the same layout as `roop fmt`.
- **An outline** (`textDocument/documentSymbol`) of the functions, structs, enums
  and sessions.
- Documents are synchronized whole on every change; roop files are small.

Any client that can run a command for a language works. In a client with a
settings file for servers, point it at `roop lsp` for files ending in `.roop`.

What is still missing is what needs the program resolved across files: checker
errors in files that use modules, go to definition through `use` and `mod` (the
resolution is `roop-modules`'s), hover, and semantic tokens. `roop test` and
`roop lean` report per function, which would suit code lenses. And the Zed
extension does not start the server by itself yet: that takes a small Rust part
built to `wasm32-wasip2` with the `zed_extension_api` crate, which tells Zed to
run `roop lsp`. The commented `[language_servers.roop-lsp]` block in
`zed/extension.toml` and the `language_servers` line in
`zed/languages/roop/config.toml` are where it plugs in.
