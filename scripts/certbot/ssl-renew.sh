#!/bin/bash

set -e

cd "/home/$USER/bible.rs"

docker-compose run --rm certbot renew && docker-compose kill -s SIGHUP gateway
