#!/usr/bin/env bash
set -eEuo pipefail

# Check whether upstream repos for dialects have updates

sed=gsed
script_dir="$(realpath "$(dirname "${BASH_SOURCE[0]}")")"

for f in "$script_dir"/*.md; do
  filename="${f#"$script_dir/"}"
  local="$("$sed" -En '/last updated .+\bgithub.com\b/ { s,.*\bhttps://github.com/([^/ ]+/[^/ ]+)/(commit|tree|blob)/([0-9a-f]+)\b.*,\1 \3,; p }' "$f")"
  [[ -z "$local" ]] && continue
  repo="${local% *}"
  repo_url="https://github.com/$repo"
  local_commit="${local##* }"
  remote="$(git ls-remote "$repo_url" HEAD)"
  remote_commit="${remote%$'\t'HEAD}"
  if [[ "$local_commit" != "$remote_commit" ]]; then
    curl -s "https://api.github.com/repos/$repo/commits/$remote_commit" |
      jq -r --arg filename "$filename" --arg repo "$repo_url" --arg local_commit "$local_commit" '
        def pad($width): . + " " * ($width - length);
        [
          ($filename | pad(20)),
          .commit.committer.date,
          (.commit.message | capture("^\\s*(?<subject>[^\n]*?)\\s*(?:\n|$)").subject | pad(50)),
          "\($repo)/compare/\($local_commit)...\(.sha)"
        ] | join(" ")
      '
  fi
done
