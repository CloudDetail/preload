# Preload

用于挂载操作系统的`execve`函数,自动加载APM探针

功能包括包括:

- 识别后续启动程序的语言类型
- 检查语言对应的APM探针并使用注入需要的环境变量
- (当前仅JAVA) 自动创建ServiceName
- 继续程序启动流程

使用 `make dist` 创建安装文件, 执行 `bash install.sh` 安装到本地

使用 `make release` 创建 `install-apo-instrument.tar.gz` 安装包, 用于在其他机器上安装

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
