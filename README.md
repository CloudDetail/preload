# Preload

用于挂载操作系统的`execve`函数,自动加载APM探针

功能包括包括:

- 识别后续启动程序的语言类型
- 检查语言对应的APM探针并使用注入需要的环境变量
- (当前仅JAVA) 自动创建ServiceName
- 继续程序启动流程

使用 `make dist` 创建安装文件, 执行 `bash install.sh` 安装到本地

使用 `make release` 创建 `install-apo-instrument.tar.gz` 安装包, 用于在其他机器上安装
