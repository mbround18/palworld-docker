#!/usr/bin/env bash


function palworld_install() {
  echo  "installing palworld"
  steamcmd +force_install_dir "/home/steam/palworld" +login anonymous  +app_update 2394010 validate +quit
  echo "palworld installed"
}

function palworld_update() {
    palworld_install
}

function palworld_configure() {
    python3 /home/steam/scripts/config.py --output /home/steam/palworld/Pal/Saved/Config/LinuxServer/PalWorldSettings.ini
}

function palworld_launch() {
  palworld_install

  cd ~/palworld || exit 1

  START_COMMAND="/home/steam/palworld/PalServer.sh"

  if [ "${COMMUNITY}" = true ]; then
      START_COMMAND="${START_COMMAND} EpicApp=PalServer"
  fi

  if [ -n "${PUBLIC_IP}" ]; then
      START_COMMAND="${START_COMMAND} -publicip=\"${PUBLIC_IP}\""
  fi

  if [ -n "${PUBLIC_PORT:-"8211"}" ]; then
      START_COMMAND="${START_COMMAND} -publiport=${PUBLIC_PORT:"8211"}"
  fi

  if [ -n "${SERVER_NAME:-"My PalWorld Server"}" ]; then
      START_COMMAND="${START_COMMAND} -servername=\"${SERVER_NAME:-"My PalWorld Server"}\""
  fi

  if [ -n "${SERVER_PASSWORD}" ]; then
      START_COMMAND="${START_COMMAND} -serverpassword=\"${SERVER_PASSWORD}\""
  fi

  if [ -n "${ADMIN_PASSWORD}" ]; then
      START_COMMAND="${START_COMMAND} -adminpassword=\"${ADMIN_PASSWORD}\""
  fi

  if [ "${MULTITHREADING}" = true ]; then
      START_COMMAND="${START_COMMAND} -useperfthreads -NoAsyncLoadingThread -UseMultithreadForDS"
  fi

  palworld_configure

  eval "bash ${START_COMMAND}"
}
