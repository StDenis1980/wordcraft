#!/usr/bin/env bash
# Refresh tinta-words/<lang>.txt from the words Tinta's server accepted
# (https://report.tintaoffice.com/v1/words/<lang>.txt), before a release. Review the diff and
# commit it: the packages ship the committed lists, so a build never depends on the server.
#
# Usage: packaging/dictionaries/update-tinta-words.sh
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
URL="${TINTA_WORDS_URL:-https://report.tintaoffice.com}"
for lang in es ca gl eu uk; do
  out="$HERE/tinta-words/$lang.txt"
  tmp="$out.download"
  curl --proto '=https' -fsSL --retry 3 -o "$tmp" "$URL/v1/words/$lang.txt"
  {
    printf '# Tinta Office: words added by Tinta Texto users and accepted by report.tintaoffice.com (%s).\n' "$lang"
    printf '# Refreshed before each release with update-tinta-words.sh; CC0 1.0.\n'
    grep -v '^#' "$tmp" | sed '/^[[:space:]]*$/d' | LC_ALL=C sort -u
  } >"$out"
  rm -f "$tmp"
  echo "$lang: $(grep -vc '^#' "$out" || true) words"
done
