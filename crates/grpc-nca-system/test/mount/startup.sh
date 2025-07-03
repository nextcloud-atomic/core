#!/usr/bin/bash

set -eux

echo 'NCA_SYSTEM_SOCKET=/run/ncatomic/nca-system.sock' >> /etc/environment
mkdir -p /etc/ncatomic/credentials /etc/ncatomic/nc-aio/credentials
#cp /mnt/testfiles/nca-system /usr/local/bin/
#chmod +x /usr/local/bin/nca-system
cp /host/nca-system/*.socket /etc/systemd/system/
cp /host/nca-system/*.service /etc/systemd/system/
systemctl daemon-reload
systemctl start nca-system.socket
