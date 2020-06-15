#!/bin/bash

function make_core_params() {
  case $3 in
  static)
    time_mode="--static-time"
    ;;
  wallclock)
    time_mode="--wall-clock-time"
    ;;
  esac
  echo "--address $1 --port $2 $time_mode --ledgerid $4"
}

function make_tls_params() {
  echo "--crt .tls_certs/end.fullchain --pem .tls_certs/end.key --client-auth ${1}"
}

function make_auth_params() {
  case $1 in
  es256)
    echo "--auth-jwt-es256-crt .auth_certs/es256.cert"
    ;;
  rs256)
    echo "--auth-jwt-rs256-crt .auth_certs/rs256.cert"
    ;;
  hs256-unsafe)
    echo "--auth-jwt-hs256-unsafe testsecret"
    ;;
  esac
}

function make_logging_params() {
  case $1 in
  info)
    echo "--log-level info"
    ;;
  debug)
    echo "--log-level debug"
    ;;
  trace)
    echo "--log-level trace"
    ;;
  esac
}

function make_seeding_params() {
  echo "--contract-id-seeding=$1"
}

function make_database_params() {
  database_name="$1_sandbox"
  echo "--jdbcurl \"jdbc:postgresql://localhost:54320/${database_name}?user=postgres\""
}

function make_early_access_params() {
  echo "--early-access-unsafe"
}

SANDBOX_TLS_ENABLED=false
SANDBOX_DATABASE_ENABLED=false
SANDBOX_EARLY_ACCESS_ENABLED=false
SANDBOX_SEEDING_MODE="testing-weak"
SANDBOX_CLIENT_AUTH="none"

while getopts "h:p:i:m:a:s:c:tdl:e" opt; do
  case ${opt} in
    h )
      SANDBOX_HOST=$OPTARG
      ;;
    p )
      SANDBOX_PORT=$OPTARG
      ;;
    i )
      SANDBOX_LEDGER_ID=$OPTARG
      ;;
    m )
      SANDBOX_TIME_MODE=$OPTARG
      ;;
    a )
      SANDBOX_AUTH_MODE=$OPTARG
      ;;
    s )
      SANDBOX_SEEDING_MODE=$OPTARG
      ;;
    c )
      SANDBOX_CLIENT_AUTH=$OPTARG
      ;;
    t )
      SANDBOX_TLS_ENABLED=true
      ;;
    d )
      SANDBOX_DATABASE_ENABLED=true
      ;;
    l )
      SANDBOX_LOG_LEVEL=$OPTARG
      ;;
    e )
      SANDBOX_EARLY_ACCESS_ENABLED=true
      ;;
  esac
done

if [[ ${SANDBOX_HOST} == "" ]] || [[ ${SANDBOX_PORT} == "" ]] || [[ ${SANDBOX_TIME_MODE} == "" ]]; then
    echo "usage: sandbox.sh -h <hostname> -p <port> -i <ledger_id> -m <static|wallclock> [-a <es256|rs256|hs256-unsafe>] [-s <seeding_mode>] [-c <client_auth_mode>] [-t] [-d] [-e] [-l <log_level>]"
    exit
fi

core_params=$(make_core_params ${SANDBOX_HOST} ${SANDBOX_PORT} ${SANDBOX_TIME_MODE} ${SANDBOX_LEDGER_ID})
if [[ ${SANDBOX_TLS_ENABLED} == true ]]; then
  tls_params=$(make_tls_params ${SANDBOX_CLIENT_AUTH})
fi
auth_params=$(make_auth_params $SANDBOX_AUTH_MODE)
logging_params=$(make_logging_params $SANDBOX_LOG_LEVEL)

if [[ ${SANDBOX_DATABASE_ENABLED} == true ]]; then
  database_params=$(make_database_params ${SANDBOX_TIME_MODE})
fi
seeding_params=$(make_seeding_params ${SANDBOX_SEEDING_MODE})
log_file="sandbox_${SANDBOX_LEDGER_ID}.log"
if [[ ${SANDBOX_EARLY_ACCESS_ENABLED} == true ]]; then
  early_access_params=$(make_early_access_params)
fi

echo "nohup daml sandbox ${core_params} ${tls_params} ${auth_params} ${logging_params} ${seeding_params} ${database_params} ${early_access_params} .daml/dist/* > ${log_file} 2>&1 &"
nohup daml sandbox ${core_params} ${tls_params} ${auth_params} ${logging_params} ${seeding_params} ${database_params} ${early_access_params} .daml/dist/* > ${log_file} 2>&1 &
