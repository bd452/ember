#!/usr/bin/env bash
# Build and package the Ember runtime and demo for both Kindle ABIs.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"$REPO_ROOT/apps/com.bd452.ember/build.sh"
"$REPO_ROOT/apps/com.bd452.emberdemo/build.sh"
"$REPO_ROOT/scripts/verify-package-reproducibility.sh"
