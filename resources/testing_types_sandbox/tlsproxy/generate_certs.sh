#!/bin/bash

rm -rf certs/
mkdir -p certs/

# Create the (self signed) CA cert (and private key)
openssl req -nodes \
          -x509 \
          -days 3650 \
          -newkey rsa:4096 \
          -keyout certs/ca.key \
          -out certs/ca.cert \
          -sha256 \
          -batch \
          -subj "/CN=Sandbox RSA CA"

# Create the intermediate CSR (and private key)
openssl req -nodes \
          -newkey rsa:3072 \
          -keyout certs/inter.key \
          -out certs/inter.req \
          -sha256 \
          -batch \
          -subj "/CN=Sandbox RSA level 2 intermediate"

# Create the end CSR (and private key)
openssl req -nodes \
          -newkey rsa:2048 \
          -keyout certs/end.key \
          -out certs/end.req \
          -sha256 \
          -batch \
          -subj "/CN=localhost"

# Create the intermediate cert (sign with CA key)
openssl x509 -req \
          -in certs/inter.req \
          -out certs/inter.cert \
          -CA certs/ca.cert \
          -CAkey certs/ca.key \
          -sha256 \
          -days 3650 \
          -set_serial 1 \
          -extensions v3_inter -extfile openssl.cnf

# Create the end cert (sign with intermediate key)
openssl x509 -req \
          -in certs/end.req \
          -out certs/end.cert \
          -CA certs/inter.cert \
          -CAkey certs/inter.key \
          -sha256 \
          -days 2000 \
          -set_serial 1 \
          -extensions v3_end -extfile openssl.cnf

# Create the end full cert (end + intermediate + CA)
cat certs/end.cert certs/inter.cert certs/ca.cert > certs/end.fullchain


