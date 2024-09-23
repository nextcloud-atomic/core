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
    origins localhost 127.0.0.1
  }
  persist_config off
}

localhost:443 {
  bind 0.0.0.0
  tls internal {
    on_demand
  }
  respond / "hello from caddy" 200
}
EOF
set -x
echo "admin socket: $CADDY_ADMIN_SOCKET"
export CADDY_ADMIN_SOCKET
docker network create ncp-test || true
docker run --user "0:$(id -g)" --rm -p 127.0.0.1:443:443 -d --name caddy --network ncp-test --cap-add=NET_ADMIN -v "$TMPDIR"/Caddyfile:/etc/caddy/Caddyfile:ro -v "$TMPDIR/caddy":/run/caddy caddy:2 caddy run --environ -c /etc/caddy/Caddyfile
sleep 1
docker exec caddy chmod g+rwx /run/caddy/admin.sock
curl --unix-socket "$CADDY_ADMIN_SOCKET" http://127.0.0.1/config/
curl -k https://localhost | grep 'hello from caddy'
docker run --user "0:$(id -g)" --rm --name ncp-activation --network ncp-test -v "$TMPDIR/caddy/admin.sock":/run/caddy/caddy-admin.sock thecalcaholic/ncp-activation:embedded
#cargo test --package ncp-core --lib caddy::tests::"$testfn" --features ssr -- --exact
#docker stop caddy
#sleep 1

#echo "SUCCESS"

