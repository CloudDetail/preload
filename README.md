# Preload

用于挂载操作系统的`execve`函数,自动加载APM探针

功能包括包括:

- 识别后续启动程序的语言类型
- 检查语言对应的APM探针并使用注入需要的环境变量
- (当前仅JAVA) 自动创建ServiceName
- 继续程序启动流程

使用 `make dist` 创建安装文件, 执行 `bash install.sh` 安装到本地

使用 `make release` 创建 `install-apo-instrument.tar.gz` 安装包, 用于在其他机器上安装

卸载方式:

从 /etc/ld.so.preload 中移除 /etc/apo/instrument/libapolanucher.so 一行;
如果只有这一行,也可以直接移除 /etc/ld.so.preload 文件;
重启命令行后完全卸载.

**特殊情况**
因为preload会在ssh登陆时加载,如果出现了ssh无法登录的情况,可以用scp拷贝空白文件来覆盖目标机器的/etc/ld.so.preload; 也可以完成卸载

随后可以通过下面的命令清理剩余文件

rm -r /etc/apo

## 在虚拟机上使用

执行 install.sh 结束后即对所有程序的启动命令生效

## 在Docker容器内使用

在宿主机上完成安装后,启动容器时添加下面的参数

    -v /etc/apo:/etc/apo
    -e LD_PRELOAD=/etc/apo/instrument/libapolanucher.so

其中 -v 用于挂载探针文件, -e 用于加载Preload库

例如原始启动命令为:

    docker run -d exampleApp:tag

修改成:

    docker run -d \
        -v /etc/apo:/etc/apo \
        -e LD_PRELOAD=/etc/apo/instrument/libapolanucher.so \
        exampleApp:tag
