#!/bin/bash
set -e

if [ ! -d "/etc/apo" ]; then
    echo "/etc/apo文件夹不存在, 虚拟机所需文件构建异常, 请重新构建镜像"
    exit 1
fi

# 移除宿主机上的/etc/ld.so.preload文件
# 移除现有的库包
if [ -f "/host/etc/ld.so.preload" ]; then
    mv /host/etc/ld.so.preload /host/etc/ld.so.preload.bak
fi
rm -f /host/etc/apo/instrument/libapoinstrument.so
rm -f /host/etc/apo/instrument/libapolanucher.so

# 拷贝最新的instrumentations包
mkdir -p /host/etc/apo/instrument
cp -rf /instrumentations /host/etc/apo/instrumentations

if [[ "$JAVA_AGENT_TYPE" == "SKYWALKING" ]]; then
    echo "检测到环境变量 JAVA_AGENT_TYPE = SKYWALKING , 加载skywalking的配置文件"
    ini-merger /etc/apo/instrument/skywalking-java/libapoinstrument.conf /etc/apo/instrument/libapoinstrument.conf
fi

if [ -z "$APO_DISABLE_CUSTOM_AGNET" ] && [ -f "/instrumentations/custom/libapoinstrument.conf" ]; then
    echo "检测到用户自定义探针, 加载自定义探针配置文件"
    ini-merger /instrumentations/custom/libapoinstrument.conf /etc/apo/instrument/libapoinstrument.conf
fi

# 加载环境变量以更新配置文件
ini-merger /etc/apo/instrument/libapoinstrument.conf

# 拷贝新的instrument库
cp -rf /etc/apo/instrument/* /host/etc/apo/instrument/

# 拷贝新的/etc/ld.so.preload
cp /host/etc/apo/instrument/apo.ld.so.preload /host/etc/ld.so.preload

echo "apo-preload 安装完成"