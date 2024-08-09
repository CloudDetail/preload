#!/bin/bash

# 下载探针文件到指定目录
mkdir -p v2.5.0
if [ ! -f v2.5.0/opentelemetry-javaagent.jar ]; then
    curl -o v2.5.0/opentelemetry-javaagent.jar -L \
        https://github.com/open-telemetry/opentelemetry-java-instrumentation/releases/download/v2.5.0/opentelemetry-javaagent.jar
fi