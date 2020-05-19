#!/bin/bash

set -e

SSH_DIR="$HOME/.ssh"
ID_RSA_FILE="$SSH_DIR/id_rsa"

echo "Installing private key"
mkdir -p $SSH_DIR
printf '%s' "$SSH_PRIVATE_KEY" > $ID_RSA_FILE
chmod 600 $ID_RSA_FILE
eval $(ssh-agent)
ssh-add $ID_RSA_FILE

echo "Installing public key of remote host"
printf '%s %s\n' "$SSH_KNOWN_HOSTS" >> "$SSH_DIR/known_hosts"
