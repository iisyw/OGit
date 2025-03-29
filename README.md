# PushGit

一个用Rust编写的Git提交和日志管理工具，可以帮助你轻松提交代码到Git仓库并自动记录开发日志。

## 功能

- 提交代码到Git仓库
- 自动记录每日开发日志
- 支持CI构建控制（可选添加[skip ci]标记）
- 智能处理日志文件日期变更
- 命令行交互式操作

## 安装

确保你已安装Rust环境，然后克隆此仓库并编译：

```bash
git clone https://github.com/yourusername/pushgit.git
cd pushgit
cargo build --release
```

编译后的可执行文件在`target/release`目录中。

## 使用方法

### 基本用法

```bash
pushgit
```

这将启动完全交互式提示，依次询问：
- 提交标注
- 是否需要推送到远程仓库
- 远程仓库名称
- 是否需要进行CI构建

也可以直接指定提交标注：

```bash
pushgit "提交消息"
```

### 命令行选项

```bash
# 提交并推送到默认远程仓库(github)
pushgit "提交消息" -p

# 提交并推送到指定远程仓库
pushgit "提交消息" -p -r origin

# 启用CI构建
pushgit "提交消息" -p -c

# 禁用CI构建(添加[skip ci]标记)
pushgit "提交消息" -p -n
# 或者使用别名
pushgit "提交消息" -p --nc
```

### 命令行参数

- `提交消息`: 提交的说明文字，如不提供则会交互式询问
- `-p, --push`: 是否推送到远程仓库
- `-r, --remote <REMOTE>`: 远程仓库名称，默认为"github"
- `-c, --ci`: 启用CI构建
- `-n, --no-ci, --nc`: 禁用CI构建，添加[skip ci]标记

## 日志文件

该工具会创建和维护两个Markdown格式的日志文件：

- `TodayDevelopment.md`: 记录当天的开发日志
- `Development.md`: 长期开发日志，当日期变更时会自动合并今日日志

## 许可证

MIT 