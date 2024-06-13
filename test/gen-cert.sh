#!/bin/bash

# Script to generate certificates

script_path="$(dirname "$(readlink -f "$0")")"
parent_path="$(dirname "$script_path")"
cd $parent_path

ca_subject="/C=US/ST=California/L=San Francisco/O=Your Company/OU=Your Department/CN=yourdomain.com/emailAddress=admin@yourdomain.com"
server_csr_subject="/C=US/ST=California/L=San Francisco/O=Your Company/OU=Your Department/CN=localhost/emailAddress=admin@yourdomain.com"
client_csr_subject="/C=US/ST=California/L=San Francisco/O=Your Company/OU=Your Department/CN=client/emailAddress=admin@yourdomain.com"
cert_ext="subjectAltName=DNS:localhost,IP:127.0.0.1"

if ! command -v openssl >/dev/null 2>&1; then
    echo "Install 'openssl' and try again"
    exit 1
fi

# Create CA Certificate and Key
openssl req -new -newkey rsa:2048 -days 365 -nodes -x509 -subj "${ca_subject}" -keyout .ca-key.pem -out .ca.pem

# Create Server Key and Certificate Signed by CA
openssl req -new -newkey rsa:2048 -nodes -keyout .server-key.pem -out .server.csr -subj "${server_csr_subject}"
openssl x509 -req -in .server.csr -CA .ca.pem -CAkey .ca-key.pem -CAcreateserial -out .server.pem -days 365 -extfile <(printf "${cert_ext}")

# Create Client Key and Certificate Signed by CA
openssl req -new -newkey rsa:2048 -nodes -keyout .client-key.pem -out .client.csr -subj "${client_csr_subject}"
openssl x509 -req -in .client.csr -CA .ca.pem -CAkey .ca-key.pem -CAcreateserial -out .client.pem -days 365 -extfile <(printf "${cert_ext}")

rm -f .ca.srl .server.csr .client.csr
