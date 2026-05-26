#!/bin/bash
set -e

if [ ! -d "/etc/apo" ]; then
    echo "/etc/apo文件夹不存在, 虚拟机所需文件构建异常, 请重新构建镜像"
    exit 1
fi

install_ld_so_preload() {
    local preload_file="/host/etc/ld.so.preload"
    local entry_file="/host/etc/apo/instrument/apo.ld.so.preload"
    local tmp_file="${preload_file}.apo.tmp.$$"
    local preload_entry

    if [ ! -f "$entry_file" ]; then
        echo "$entry_file 不存在, 无法更新 $preload_file"
        exit 1
    fi

    preload_entry="$(grep -v '^[[:space:]]*$' "$entry_file" | head -n 1)"
    if [ -z "$preload_entry" ]; then
        echo "$entry_file 内容为空, 无法更新 $preload_file"
        exit 1
    fi

    if [ -f "$preload_file" ] && grep -Fxq "$preload_entry" "$preload_file"; then
        echo "$preload_file 已包含 apo preload 配置"
        return
    fi

    if [ -f "$preload_file" ]; then
        cp "$preload_file" "$tmp_file"
        if [ -s "$tmp_file" ] && [ "$(tail -c 1 "$tmp_file")" != "" ]; then
            printf '\n' >> "$tmp_file"
        fi
    else
        : > "$tmp_file"
    fi

    printf '%s\n' "$preload_entry" >> "$tmp_file"
    chmod 755 "$tmp_file"
    mv -f "$tmp_file" "$preload_file"
}

# 移除现有的库包
rm -f /host/etc/apo/instrument/libapoinstrument.so
rm -f /host/etc/apo/instrument/libapoinstrument_musl.so
rm -f /host/etc/apo/instrument/libapolanucher.so

# 拷贝最新的instrumentations包
mkdir -p /host/etc/apo/instrumentations
rm -rf /host/etc/apo/instrumentations/dotnet
rm -rf /host/etc/apo/instrumentations/java
rm -rf /host/etc/apo/instrumentations/nodejs
rm -rf /host/etc/apo/instrumentations/python
rm -rf /host/etc/apo/instrumentations/skywalking
rm -rf /host/etc/apo/instrumentations/custom
cp -r /instrumentations/* /host/etc/apo/instrumentations

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
mkdir -p /host/etc/apo/instrument
cp -rf /etc/apo/instrument/* /host/etc/apo/instrument/
chmod -R 755 /host/etc/apo

# 检查并插入/etc/ld.so.preload, 保留已有配置
install_ld_so_preload

echo "apo-preload 安装完成"
