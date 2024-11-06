#!/bin/sh

env > /etc/environment
ssh-keygen -A
exec /usr/sbin/sshd -D -e "$@"