#!/usr/bin/env bash
# Fetches the CypherBench property graphs into the gitignored data
# directory. Small artifacts (schemas, test.json) are vendored in git;
# the graphs are too large (full tier totals ~5.2GB, biology alone
# 2.4GB) and download on demand at a pinned revision.
#
# Usage:
#   scripts/fetch-cypherbench.sh sampled   # ~56MB, enough for bench-sample
#   scripts/fetch-cypherbench.sh full      # ~5.2GB, needed for bench-full
#   scripts/fetch-cypherbench.sh all       # both tiers plus train.json
set -euo pipefail

REVISION="efdfde14c04fe174b4960544c1b1001530e2a178"
BASE="https://huggingface.co/datasets/megagonlabs/cypherbench/resolve/${REVISION}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DATA="${ROOT}/graph/testdata/benchmarks/cypherbench/data"
DOMAINS=(art biology company fictional_character flight_accident geography movie nba politics soccer terrorist_attack)

tier="${1:-sampled}"

fetch() {
    local url="$1" target="$2"
    if [[ -s "$target" ]]; then
        echo "have    ${target#"$ROOT"/}"
        return
    fi
    echo "fetch   ${target#"$ROOT"/}"
    mkdir -p "$(dirname "$target")"
    curl -fL --retry 3 --progress-bar "$url" -o "$target.part"
    mv "$target.part" "$target"
}

fetch_sampled() {
    for domain in "${DOMAINS[@]}"; do
        fetch "${BASE}/graphs/simplekg_sampled/${domain}_sampled_simplekg.json" \
            "${DATA}/simplekg_sampled/${domain}_sampled_simplekg.json"
    done
}

fetch_full() {
    for domain in "${DOMAINS[@]}"; do
        fetch "${BASE}/graphs/simplekg/${domain}_simplekg.json" \
            "${DATA}/simplekg/${domain}_simplekg.json"
    done
}

case "$tier" in
sampled) fetch_sampled ;;
full) fetch_full ;;
all)
    fetch_sampled
    fetch_full
    fetch "${BASE}/train.json" "${DATA}/train.json"
    ;;
*)
    echo "usage: $0 [sampled|full|all]" >&2
    exit 1
    ;;
esac
echo "done: ${DATA}"
