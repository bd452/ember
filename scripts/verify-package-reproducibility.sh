#!/usr/bin/env bash
# Verify both archives and prove that repacking their staged trees is byte-stable.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

for package_id in com.bd452.ember com.bd452.emberdemo; do
    app="$REPO_ROOT/apps/$package_id"
    artifact=""
    for candidate in "$app/dist/$package_id"_*.kpkg; do
        [[ -e "$candidate" ]] || continue
        [[ -z "$artifact" ]] || {
            echo "error: multiple artifacts found for $package_id" >&2
            exit 1
        }
        artifact="$candidate"
    done
    [[ -n "$artifact" ]] || {
        echo "error: missing artifact for $package_id" >&2
        exit 1
    }

    "$REPO_ROOT/scripts/kpm-dev" verify "$artifact"
    output="$TMP_DIR/$package_id"
    mkdir -p "$output"
    "$REPO_ROOT/scripts/kpm-dev" pack "$app/dist/pkg" --output "$output"
    rebuilt="$output/$(basename "$artifact")"
    cmp "$artifact" "$rebuilt"
    echo "==> Reproducible: $(basename "$artifact")"
done
