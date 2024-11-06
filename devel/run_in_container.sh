#!/usr/bin/env bash

set -ex

SSH_OPTS=(-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -p 2222 -i ./devel/ssh_key)

files=()
if compgen -G "./*"; then
    files+=(./*)
fi
if compgen -G "./.[!.]*"; then
    files+=(./.[!.]*)
fi
rsync -rvP --delete \
  --exclude ".dioxus" --exclude "target" --exclude "dist" --exclude "resource" \
  -e "ssh ${SSH_OPTS[*]}" \
  "${files[@]}" root@nextcloudatomic.local:/app/

files=()
if compgen -G "./resource/*"; then
    files+=(./resource/*)
fi
if compgen -G "./resource/.[!.]*"; then
    files+=(./resource/.[!.]*)
fi
rsync -rvP --delete \
  -e "ssh ${SSH_OPTS[*]}" \
  "${files[@]}" root@nextcloudatomic.local:/resource/
ssh "${SSH_OPTS[@]}" root@nextcloudatomic.local ". /etc/environment; cd /app/ && /usr/local/cargo/bin/dx build --platform fullstack --release --target=x86_64-unknown-linux-musl"
ssh "${SSH_OPTS[@]}" root@nextcloudatomic.local bash <<EOF
set -e
. /etc/environment

export NCA_CONFIG_SOURCE=/resource/templates
export NCA_CONFIG_TARGET=/etc/ncatomic
export DIOXUS_IP="0.0.0.0"
export CADDY_ADMIN_SOCKET

cd /app && /app/dist/activate
EOF
