#!/bin/bash

rm -rf .tls_certs/
mkdir -p .tls_certs/

#
# Generate certificates and keys for TLS connectivity
#

# Create the (self signed) CA cert (and private key)
openssl req -nodes \
          -x509 \
          -days 3650 \
          -newkey rsa:4096 \
          -keyout .tls_certs/ca.key \
          -out .tls_certs/ca.cert \
          -sha256 \
          -batch \
          -subj "/CN=Sandbox RSA CA"

# Create the intermediate CSR (and private key)
openssl req -nodes \
          -newkey rsa:3072 \
          -keyout .tls_certs/inter.key \
          -out .tls_certs/inter.req \
          -sha256 \
          -batch \
          -subj "/CN=Sandbox RSA level 2 intermediate"

# Create the end CSR (and private key)
openssl req -nodes \
          -newkey rsa:2048 \
          -keyout .tls_certs/end.key \
          -out .tls_certs/end.req \
          -sha256 \
          -batch \
          -subj "/CN=localhost"

# Create the intermediate cert (sign with CA key)
openssl x509 -req \
          -in .tls_certs/inter.req \
          -out .tls_certs/inter.cert \
          -CA .tls_certs/ca.cert \
          -CAkey .tls_certs/ca.key \
          -sha256 \
          -days 3650 \
          -set_serial 1 \
          -extensions v3_inter -extfile openssl.cnf

# Create the end cert (sign with intermediate key)
openssl x509 -req \
          -in .tls_certs/end.req \
          -out .tls_certs/end.cert \
          -CA .tls_certs/inter.cert \
          -CAkey .tls_certs/inter.key \
          -sha256 \
          -days 2000 \
          -set_serial 1 \
          -extensions v3_end -extfile openssl.cnf

# Create the end full cert (end + intermediate + CA)
cat .tls_certs/end.cert .tls_certs/inter.cert .tls_certs/ca.cert > .tls_certs/end.fullchain

#
# Generate keys certificates for token authentication
#

rm -rf .auth_certs/
mkdir -p .auth_certs/

# Create the RSA256 cert and private key
openssl req -nodes \
          -new -x509 \
          -keyout .auth_certs/rs256.key \
          -out .auth_certs/rs256.cert \
          -batch \
          -subj "/CN=Sandbox RSA256"

# Create the es256 cert and private key
openssl req \
          -x509 \
          -nodes \
          -days 3650 \
          -newkey ec:<(openssl ecparam -name prime256v1) \
          -keyout .auth_certs/es256.key \
          -out .auth_certs/es256.cert \
          -batch \
          -subj "/CN=Sandbox ECDSA256 CA"
