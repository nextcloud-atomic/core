#!/bin/bash

set -e
trap 'if [[ "$?" -ne 0 ]]; then echo "CADDY LOGS:"; docker logs caddy; echo "=========="; fi; docker inspect caddy > /dev/null 2>&1 && docker stop caddy || true; rm -fr "$TMPDIR"' EXIT
TMPDIR="$(mktemp -d)"
mkdir -p "$TMPDIR/caddy"
CADDY_ADMIN_SOCKET="$TMPDIR/caddy/admin.sock"
touch "$CADDY_ADMIN_SOCKET"
chmod ugo+rwx "$TMPDIR"/caddy/admin.sock
cat <<'EOF' > "$TMPDIR"/Caddyfile
{
  admin unix//run/caddy/admin.sock {
    origins localhost
  }
  persist_config off
}

localhost:80 {
  bind 0.0.0.0
  header Content-Type text/plain
  respond / "site1" 200
}
EOF

echo "admin socket: $CADDY_ADMIN_SOCKET"
export CADDY_ADMIN_SOCKET
for testfn in 'test_set_static_response' 'test_change_caddy_config'
do
  docker run --user "0:$(id -g)" --rm -p 127.0.0.1:80:80 -d --name caddy --cap-add=NET_ADMIN -v "$TMPDIR"/Caddyfile:/etc/caddy/Caddyfile:ro -v "$TMPDIR/caddy":/run/caddy caddy:2 caddy run --environ -c /etc/caddy/Caddyfile
  sleep 1
  docker exec caddy chmod g+rwx /run/caddy/admin.sock
  curl --unix-socket "$CADDY_ADMIN_SOCKET" http://127.0.0.1/config/ > /dev/null
  curl http://localhost | grep 'site1'
  cargo test --package ncp-core --lib caddy::tests::"$testfn" --features server -- --exact
  docker stop caddy
  sleep 1
done

echo "SUCCESS"

