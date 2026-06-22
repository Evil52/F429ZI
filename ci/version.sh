#!/usr/bin/env bash
# ci/version.sh — emit a build-identity line for traceability (IEC 62304).
#
# Every CI stage saves version.txt as an artifact so any produced .elf/report can
# be traced back to an exact commit + build time. Mirrors the C pipeline's
# ci/version.sh. Output goes to stdout; the workflow redirects it to version.txt.
#
# Fields:
#   describe : `git describe` (nearest tag + commits-ahead + short sha), or the
#              short sha if there are no tags yet.
#   commit   : full commit sha (immutable build identity)
#   branch   : branch name (or detached)
#   built    : UTC timestamp (reproducibility audit trail)
set -euo pipefail

describe="$(git describe --tags --always --dirty 2>/dev/null || git rev-parse --short HEAD)"
commit="$(git rev-parse HEAD 2>/dev/null || echo unknown)"
branch="${CI_COMMIT_BRANCH:-$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)}"
built="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cat <<EOF
project : f429zi (NUCLEO-F429ZI, Rust + Embassy)
describe: ${describe}
commit  : ${commit}
branch  : ${branch}
built   : ${built}
EOF
