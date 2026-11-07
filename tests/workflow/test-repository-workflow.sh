#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

# Exercise installed tools and inspect their instruction/configuration contract.
# This suite does not simulate an agent or claim workflow compliance.
python3 "$root/tests/workflow/test-onboarding-evidence.py"

echo "installed repository context, automatic skill policies, and evidence contracts passed; agent behavior not tested"
