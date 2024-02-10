#!/bin/bash

echo "Setting up SSH keys"
ssh-keygen -t rsa -b 4096 -f ~/.ssh/id_rsa -N ''
cp ~/.ssh/id_rsa.pub ~/.ssh/authorized_keys
chmod 600 ~/.ssh/authorized_keys ~/.ssh/id_rsa
echo -e "Host localhost\n\tStrictHostKeyChecking no\n\tUserKnownHostsFile=/dev/null" > ~/.ssh/config
chmod 600 ~/.ssh/config
echo "SSH keys setup complete"