#!/bin/bash

while getopts "h:p:b:" opt; do
  case ${opt} in
  h)
    SANDBOX_HOST=$OPTARG
    ;;
  p)
    SANDBOX_PORT=$OPTARG
    ;;
  b)
    JSON_BRIDGE_PORT=$OPTARG
    ;;
  esac
done

if [[ ${SANDBOX_HOST} == "" ]] || [[ ${SANDBOX_PORT} == "" ]] || [[ ${JSON_BRIDGE_PORT} == "" ]]; then
  echo "usage: json_bridge.sh -h <hostname> -p <port> -b <port>"
  exit
fi

if [[ -e "json-api.log" ]]; then
  rm json-api.log
fi

# attempt to start the json-api in the background, retrying if it fails (it fails if the sandbox isn't available)
(
  until nohup daml json-api --ledger-host ${SANDBOX_HOST} --ledger-port ${SANDBOX_PORT} --http-port ${JSON_BRIDGE_PORT} --allow-insecure-tokens --package-reload-interval 1d >>json-api.log 2>&1; do
    echo "retrying..." >>json-api.log
    sleep 1
  done
) &

# wait for the json-api to complete startup
echo "waiting for JSON API to startup..."
until grep "Started server" json-api.log > /dev/null ; do
    sleep 5
done
