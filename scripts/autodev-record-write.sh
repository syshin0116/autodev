#!/usr/bin/env bash
# Writes the durable authorization record for one issue.
#
# Usage: autodev-record-write.sh <owner/repo> <issue> <comment-id|""> <authorizations.json>
set -euo pipefail

exec "$(dirname "${BASH_SOURCE[0]}")/autodev-comment-record.sh" write \
  "${1:?owner/repo}" "${2:?issue number}" "${3-}" '<!-- autodev:authorization -->' \
  "${4:?authorizations json path}"
