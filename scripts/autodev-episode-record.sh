#!/usr/bin/env bash
# Reads the durable authorization record for one issue.
#
# Prints {"comment_id": <id|null>, "authorizations": []} with an empty list when
# the issue has no record yet. See autodev-comment-record.sh for the format and
# the test seams.
#
# Usage: autodev-episode-record.sh <owner/repo> <issue-number>
set -euo pipefail

exec "$(dirname "${BASH_SOURCE[0]}")/autodev-comment-record.sh" read \
  "${1:?owner/repo}" "${2:?issue number}" '<!-- autodev:authorization -->' authorizations
