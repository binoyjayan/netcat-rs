#!/bin/bash

# Script to generate certificates using 'openssl'

script_path="$(dirname "$(readlink -f "$0")")"
cd $script_path

ca_subject="/C=US/ST=California/L=San Francisco/O=Your Company/OU=Your Department/CN=yourdomain.com/emailAddress=admin@yourdomain.com"
csr_subject="/C=US/ST=California/L=San Francisco/O=Your Company/OU=Your Department/CN=localhost/emailAddress=admin@yourdomain.com"
cert_ext="subjectAltName=DNS:localhost,IP:127.0.0.1"

if ! command -v openssl >/dev/null 2>&1; then
    echo "Install 'openssl' and try again"
    exit
fi

openssl req -new -newkey rsa:2048 -days 365 -nodes -x509 -subj "${ca_subject}" -keyout .ca-key.pem -out .ca.pem
openssl req -new -newkey rsa:2048 -nodes -keyout .server-key.pem -out .server.csr -subj "${csr_subject}"
openssl x509 -req -in .server.csr -CA .ca.pem -CAkey .ca-key.pem -CAcreateserial -out .server.pem -days 365 -extfile <(printf "${cert_ext}")

rm -f .server.csr .ca.srl
