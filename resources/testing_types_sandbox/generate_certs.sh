#!/bin/bash

rm -rf certs/
mkdir -p certs/

# Create the RSA256 cert and private key
openssl req -nodes \
          -new -x509 \
          -keyout certs/rs256.key \
          -out certs/rs256.cert \
          -batch \
          -subj "/CN=Sandbox RSA256"

# Create the es256 cert and private key
openssl req \
          -x509 \
          -nodes \
          -days 3650 \
          -newkey ec:<(openssl ecparam -name prime256v1) \
          -keyout certs/es256.key \
          -out certs/es256.cert \
          -batch \
          -subj "/CN=Sandbox ECDSA256 CA"
