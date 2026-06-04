#!/bin/bash

mkdir -p /opt/apps/remote-access-server

cd /opt/apps/remote-access-server

printf "APP_ID=123\nAPP_PORT=123\nAPP_HOST=ras123.gbssg.russos.ch">.env

# curl zip
# - start.sh
# - ras-rs
# - ras.service
# - docker-compose.yml (includes built frontend ghcr image)
# - users.yaml
# - Rocket.toml

tar -xvzf required-files.tar.gz -C .

cp ras.service ~/.config/systemd/user/

systemctl --user enable --now ras.service

docker compose up -d