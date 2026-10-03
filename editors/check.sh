#!/bin/sh
# Checks the editor support: the generated parser is current, the corpus
# passes, every roop file in the repository parses without an error node, and
# the queries compile against the grammar.
set -e
root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root/editors/tree-sitter-roop"
[ -d node_modules ] || npm install --no-audit --no-fund
ts=node_modules/.bin/tree-sitter
$ts generate
if ! git diff --quiet -- src; then
    echo "src/ is out of date: run tree-sitter generate and commit the result" >&2
    exit 1
fi
$ts test
for f in $(find "$root/roop" "$root/crates" -name '*.roop' -not -path '*/target/*'); do
    if $ts parse "$f" 2>&1 | grep -q 'ERROR\|MISSING'; then
        echo "parse error in $f" >&2
        exit 1
    fi
done
for q in "$root"/editors/zed/languages/roop/*.scm; do
    $ts query "$q" "$root/roop/tests/blas.roop" > /dev/null 2>&1 || {
        echo "query does not compile: $q" >&2
        exit 1
    }
done
echo "editor support is in order"
