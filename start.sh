#!/usr/bin/env bash
set -euo pipefail

export NODE_ENV="${NODE_ENV:-production}"
export PORT="${PORT:-6129}"

NODE_BIN=""
if command -v node >/dev/null 2>&1; then
  NODE_BIN="$(command -v node)"
elif [ -x "/home/bohat/.local/share/fnm/node-versions/v22.22.3/installation/bin/node" ]; then
  NODE_BIN="/home/bohat/.local/share/fnm/node-versions/v22.22.3/installation/bin/node"
else
  for candidate in /home/bohat/.local/share/fnm/node-versions/*/installation/bin/node; do
    if [ -x "$candidate" ]; then
      NODE_BIN="$candidate"
    fi
  done
fi

if [ -z "$NODE_BIN" ]; then
  echo "Error: node binary not found" >&2
  exit 1
fi

"$NODE_BIN" /home/bohat/projects/rustflow-draw/apps/web/build.mjs

exec "$NODE_BIN" /home/bohat/projects/rustflow-draw/apps/web/server.mjs
