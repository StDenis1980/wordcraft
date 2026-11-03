#!/usr/bin/env bash
# Put the Tinta Office spelling dictionaries (sources.tsv) into DEST, checking each file's SHA-256.
# Downloads are kept in TINTA_DICTIONARIES_CACHE (default: target/dictionaries-cache), so a second
# run needs no network. Used by packaging/linux/package.sh and packaging/macos/package.sh; Windows
# uses fetch.ps1.
#
# Usage: packaging/dictionaries/fetch.sh DEST [LIST]
#   LIST: the manifest in this folder, sources.tsv (the shipped pack, default) or sources-extra.tsv
#   (the separate download).
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
DEST="${1:?usage: fetch.sh DEST [LIST]}"
LIST="${2:-sources.tsv}"
case "$LIST" in sources.tsv | sources-extra.tsv) ;; *) echo "error: unknown list $LIST" >&2; exit 1 ;; esac
CACHE="${TINTA_DICTIONARIES_CACHE:-${CARGO_TARGET_DIR:-$ROOT/target}/dictionaries-cache}"
mkdir -p "$DEST/licences" "$CACHE"
# macOS has shasum, not sha256sum.
if command -v sha256sum >/dev/null; then SHA=(sha256sum); else SHA=(shasum -a 256); fi
matches() { printf '%s  %s\n' "$1" "$2" | "${SHA[@]}" -c --status -; }
while IFS=$'\t' read -r file url sha _langs _licence; do
  [[ -z "$file" || "$file" == \#* ]] && continue
  cached="$CACHE/$sha"
  if [[ ! -f "$cached" ]] || ! matches "$sha" "$cached"; then
    curl --proto '=https' -fsSL --retry 3 -o "$cached.download" "$url"
    if ! matches "$sha" "$cached.download"; then
      echo "error: $url does not match its SHA-256 in $LIST" >&2
      rm -f "$cached.download"
      exit 1
    fi
    mv "$cached.download" "$cached"
  fi
  mkdir -p "$(dirname "$DEST/$file")"
  cp "$cached" "$DEST/$file"
  chmod 644 "$DEST/$file"
done <"$HERE/$LIST"
# Tinta's accepted words (tinta-words/, committed) go beside the shipped dictionaries.
if [[ "$LIST" == sources.tsv ]]; then
  for list in "$HERE"/tinta-words/*.txt; do
    cp "$list" "$DEST/tinta-words-$(basename "$list")"
    chmod 644 "$DEST/tinta-words-$(basename "$list")"
  done
fi
cp "$HERE/README.md" "$DEST/licences/README.md"
echo "dictionaries: $(find "$DEST" -maxdepth 1 -name '*.dic' | wc -l) in $DEST"
