#!/usr/bin/env bash
# Run the sidecar demo without pytest.
# Usage: bash scripts/run_demo.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")/.." && pwd)"
SIDECAR="$HERE/sidecar/bin/realestate-extract"

if [ ! -x "$SIDECAR" ]; then
    echo "ERROR: sidecar not built. Expected: $SIDECAR" >&2
    exit 1
fi

export PYTHONPATH=

run() {
    local label="$1"
    local doc="$2"
    local schema="$3"
    echo "============================================================"
    echo "$label"
    echo "  doc:    $doc"
    echo "  schema: $schema"
    echo "============================================================"
    echo "{\"doc_path\": \"$doc\", \"schema_id\": \"$schema\"}" \
        | "$SIDECAR" \
        | python3 -c 'import json, sys; print(json.dumps(json.load(sys.stdin), indent=2))'
    echo
}

run "Invoice extraction"  "$HERE/tests/sample_invoice.txt" invoice
run "Lease extraction"    "$HERE/tests/sample_lease.txt"   lease_addendum
