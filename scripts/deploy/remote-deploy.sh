#!/bin/bash

set -e

cd "/home/$USER/bible.rs"

docker-compose pull

sudo systemctl restart biblers.service
