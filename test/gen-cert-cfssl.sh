#!/bin/bash

# Script to generate certificates using 'cfssl'

script_path="$(dirname "$(readlink -f "$0")")"
cd $script_path

if ! command -v cfssl >/dev/null 2>&1; then
    echo "Install 'cfssl' and try again"
    exit
fi

cfssl gencert -initca ca-csr.json | cfssljson -bare .ca
cfssl gencert -ca .ca.pem -ca-key .ca-key.pem -config ca-config.json -profile server server.json | cfssljson -bare .server

rm -f .ca.csr .server.csr .ca.srl
