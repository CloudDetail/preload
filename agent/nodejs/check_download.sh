#!/bin/bash
set -e

mkdir -p v0.48.0
if [ ! -d v0.48.0/package ]; then
    npm pack @opentelemetry/api@1.9.0
    tar -zxvf opentelemetry-api-1.9.0.tgz -C v0.48.0
    npm pack @opentelemetry/auto-instrumentations-node@0.48.0
    tar -zxvf opentelemetry-auto-instrumentations-node-0.48.0.tgz -C v0.48.0
fi


