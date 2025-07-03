#!/usr/bin/bash

set -eux

TEST_UUID="$(cat /proc/sys/kernel/random/uuid)"
VM="nca-system-test-${TEST_UUID?}"
WORKING_DIRECTORY="$(cd "$(dirname 0)" && echo "$PWD")"

trap 'incus rm -f $VM' HUP INT QUIT PIPE TERM

incus init --vm mkosi/ncatomic-vm "$VM" \
  -c security.secureboot=false \
  -d root,size=12GiB
#  -c security.nesting=true \
#  -c cloud-init.user-data="$(cat cloudinit.yaml)" \

incus config device add "$VM" vtpm tpm #path=/dev/tmp0 pathrm=/dev/tpmrm0
incus config device add "$VM" "build-artifacts" disk \
  source="${WORKING_DIRECTORY}/mount" \
  path=/mnt/testfiles \
  readonly=true \
  shift=true

#incus config device add "${VM}" port-80 proxy listen=tcp:127.0.0.1:80 connect=tcp:127.0.0.1:80
#incus config device add "${VM}" port-443 proxy listen=tcp:127.0.0.1:443 connect=tcp:127.0.0.1:443

incus start "${VM}" --console

for i in {1..100}
do
#  if incus exec "${VM}" -- cloud-init status 2> /dev/null | grep 'done'
#  then
#    break
#  fi
#  incus exec "$VM" -- cloud-init status || :;
  if incus exec "${VM}" -- echo 'online'
  then
    break
  fi
  sleep 3
done

incus exec "${VM}" -- bash /mnt/testfiles/startup.sh

if [[ "$i" -eq 100 ]]
then
  echo "Timeout while waiting for container startup (i.e. cloud-init to be run successfully)"
  exit 1
fi

