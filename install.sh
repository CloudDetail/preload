#!/bin/bash
set -e

if [ ! -d "apo-instrument" ]; then
    echo "apo-instrument文件夹不存在, 请执行 make dist 命令生成安装文件"
    exit 1
fi

# 移除并备份当前的/etc/ld.so.preload
# 确保本次拷贝命令不会由于/etc/ld.so.preload被修改导致拷贝失败
if [ -f "/etc/ld.so.preload" ]; then
    cp /etc/ld.so.preload .
    mv /etc/ld.so.preload /etc/ld.so.preload.bak
fi

# 拷贝最新的instrument包
mkdir -p /etc/apo/instrument
# !!! 在系统上已经存在 libapoinstrument.so/ibapolanucher.so的情况下
# !!! 永远先删除已有的库文件,然后再进行替换
rm -f /etc/apo/instrument/libapoinstrument.so
rm -f /etc/apo/instrument/libapolanucher.so
cp -rf apo-instrument/* /etc/apo/instrument/.

# 拷贝新的/etc/ld.so.preload
# 开始装载注入程序
cp /etc/apo/instrument/apo.ld.so.preload /etc/ld.so.preload
# TODO 恢复原有的ld.so.preload(注意移除自身)
# cat /etc/ld.so.preload.bak | grep -v "apoinstrument" >> /etc/ld.so.preload
