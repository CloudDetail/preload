#!/bin/bash

# 下载探针文件到指定目录
mkdir -p opentelemetry/v2.5.0
if [ ! -f opentelemetry/v2.5.0/opentelemetry-javaagent.jar ]; then
    curl -o opentelemetry/v2.5.0/opentelemetry-javaagent.jar -L \
        https://github.com/open-telemetry/opentelemetry-java-instrumentation/releases/download/v2.5.0/opentelemetry-javaagent.jar
fi

mkdir -p skywalking/
if [ ! -f skywalking/v9.3.0/skywalking-agent.jar ]; then
    curl -O https://dlcdn.apache.org/skywalking/java-agent/9.3.0/apache-skywalking-java-agent-9.3.0.tgz
    tar -zxvf apache-skywalking-java-agent-9.3.0.tgz -C skywalking
    mv skywalking/skywalking-agent skywalking/v9.3.0
    rm apache-skywalking-java-agent-9.3.0.tgz
fi