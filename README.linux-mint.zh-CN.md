# Linux Mint 长时间自动运行开发环境

这份指南针对你的环境：Linux Mint 最新版、16GB 内存、已安装 Codex CLI、Clash mixed port 为 `7897`。

目标是把本项目部署成一套稳定的长时间自动开发环境：Ralph 负责循环调度，Codex CLI 负责每轮实现一个 `prd.json` story，Clash 负责网络代理，`tmux` 或 `systemd --user` 负责长时间运行。

## 目录结构

```text
linux-mint/
  bootstrap.sh
  check-env.sh
  create-swapfile.sh
  install-systemd-user.sh
  remote-status.sh
  ralph.env.example
  run-ralph.sh
  setup-remote-access.sh
```

| 文件 | 作用 |
| --- | --- |
| `bootstrap.sh` | 幂等安装脚本：先检查，缺什么装什么 |
| `check-env.sh` | 检查全部依赖、Codex、Clash、项目和 Ralph 文件 |
| `create-swapfile.sh` | 可选：创建 swapfile，适合长时间无人值守任务 |
| `run-ralph.sh` | 读取配置、导出 Clash 代理变量、启动 Ralph |
| `install-systemd-user.sh` | 安装 systemd user service |
| `setup-remote-access.sh` | 安装/启用 SSH，并按需配置内网 Web 端口 |
| `remote-status.sh` | 通过 SSH 快速查看 Ralph、系统资源、story 和 Web 端口 |
| `ralph.env.example` | 环境变量模板 |

## 1. Bootstrap 会安装什么

执行：

```bash
cd /path/to/codex-ralph
chmod +x linux-mint/*.sh
./linux-mint/bootstrap.sh
```

`bootstrap.sh` 是幂等脚本。它会先检查软件是否已经存在：

- 已安装：输出 `[skip]`，不重复安装。
- 未安装：才执行 `sudo apt install -y ...`。
- apt 仓库中没有的可选工具：输出 warning，不阻塞基础环境。

会按需安装的软件：

| 类别 | 命令/能力 | apt 包 |
| --- | --- | --- |
| 基础工具 | `bash` | `bash` |
| 基础工具 | `git` | `git` |
| 基础工具 | `jq` | `jq` |
| 基础工具 | `curl` | `curl` |
| 基础工具 | `update-ca-certificates` | `ca-certificates` |
| 沙盒工具 | `bwrap` | `bubblewrap` |
| 终端运行 | `tmux` | `tmux` |
| 进程工具 | `pgrep` | `procps` |
| 进程工具 | `killall` | `psmisc` |
| 压缩工具 | `unzip` | `unzip` |
| 压缩工具 | `zip` | `zip` |
| 编译工具 | `gcc`、`g++`、`make` | `build-essential` |
| 编译工具 | `pkg-config` | `pkg-config` |
| Python | `python3` | `python3` |
| Python | `pip3` | `python3-pip` |
| Python | `python3 -m venv` | `python3-venv` |
| Node | `node` | `nodejs` |
| Node | `npm` | `npm` |
| 搜索工具 | `rg` | `ripgrep` |
| 搜索工具 | `fd` 或 `fdfind` | `fd-find` |
| 静态检查 | `shellcheck` | `shellcheck` |
| GitHub | `gh` | `gh`，如果 apt 仓库可用 |
| Docker | `docker` | `docker.io`，如果 apt 仓库可用 |
| Docker Compose | `docker compose` | `docker-compose-plugin`、`docker-compose-v2` 或 `docker-compose` 中可用者 |

不会自动安装：

- Codex CLI：你已经安装了，脚本只检查 `codex exec --help`。
- Clash：脚本只检查 `127.0.0.1:7897` mixed port 是否可连。

`bubblewrap` 的命令名是 `bwrap`，这是很多 Linux 环境推荐安装的沙盒 wrapper。Codex CLI 的 Linux sandbox 子命令默认会用它。当前 Ralph 默认为了无人值守自动开发使用 `--dangerously-bypass-approvals-and-sandbox`，但安装 `bwrap` 仍然有价值：你可以在需要时运行 Codex 的沙盒命令，也能让环境更接近 Codex CLI 的推荐 Linux 配置。

## 2. 16GB 内存建议

16GB 内存适合跑 Ralph + Codex CLI。建议配置：

```bash
RALPH_ITERATIONS=30
RALPH_MAX_RETRIES=3
RALPH_MAX_STORY_FAILURES=3
RALPH_RETRY_BASE_SECONDS=10
RALPH_SLEEP_SECONDS=3
```

建议保留至少 5GB 可用磁盘空间，并配置 4GB 到 8GB swap。

检查当前内存和 swap：

```bash
free -h
swapon --show
```

如果没有 swap，创建 8GB swapfile：

```bash
cd /path/to/codex-ralph
sudo ./linux-mint/create-swapfile.sh 8
```

如果你已经有 4GB 或更多 swap，可以跳过这一步。

## 3. 配置 Ralph 环境变量

bootstrap 会创建：

```bash
~/.config/ralph-codex/ralph.env
```

编辑：

```bash
nano ~/.config/ralph-codex/ralph.env
```

最重要的是：

```bash
PROJECT_ROOT="$HOME/projects/my-app"
RALPH_DIR="$PROJECT_ROOT/scripts/ralph"
```

把它们改成你的目标项目路径。

Clash 默认配置为：

```bash
RALPH_PROXY_HOST=127.0.0.1
RALPH_PROXY_PORT=7897
RALPH_HTTP_PROXY=http://127.0.0.1:7897
RALPH_ALL_PROXY=socks5h://127.0.0.1:7897
```

`run-ralph.sh` 会自动导出：

```bash
http_proxy
https_proxy
HTTP_PROXY
HTTPS_PROXY
all_proxy
ALL_PROXY
no_proxy
NO_PROXY
```

这样 Codex CLI、git HTTPS、curl、npm 等常见工具都会优先走 Clash。

## 4. 把 Ralph 放进目标项目

假设：

- 本仓库在 `~/codex-ralph`
- 目标项目在 `~/projects/my-app`

执行：

```bash
cd ~/projects/my-app
mkdir -p scripts/ralph
cp ~/codex-ralph/ralph.sh scripts/ralph/
cp ~/codex-ralph/CODEX.md scripts/ralph/
cp ~/codex-ralph/prd.json.example scripts/ralph/
chmod +x scripts/ralph/ralph.sh
```

如果目标项目还不是 Git 仓库：

```bash
git init
```

Ralph 必须在 Git 仓库里运行。

## 5. 准备 prd.json

```bash
cd ~/projects/my-app
cp scripts/ralph/prd.json.example scripts/ralph/prd.json
nano scripts/ralph/prd.json
jq . scripts/ralph/prd.json >/dev/null
```

基本规则：

- `branchName` 是 Ralph 要创建或切换的开发分支。
- `priority` 数字越小越先执行。
- `passes:false` 表示未完成。
- 每个 story 必须足够小，最好一轮 Codex 能完成。

## 6. 检查环境

执行：

```bash
cd ~/codex-ralph
./linux-mint/check-env.sh
```

检查内容包括：

- Linux Mint 系统识别。
- 内存、swap、项目磁盘空间。
- 基础工具、沙盒工具、编译工具、Node、Python、搜索工具、ShellCheck。
- GitHub CLI、Docker、Docker Compose，可选缺失会单独标记。
- OpenSSH、ufw、ss 等远程访问工具，可选缺失会单独标记。
- Codex CLI `codex exec --help`。
- Clash mixed port `127.0.0.1:7897`。
- `PROJECT_ROOT` 是否是 Git 仓库。
- `RALPH_DIR/ralph.sh`、`CODEX.md`、`prd.json` 是否可用。

如果 Docker 已安装但当前用户不在 `docker` 组，检查脚本会提示重新登录或执行：

```bash
newgrp docker
```

## 7. 直接运行

```bash
cd ~/codex-ralph
./linux-mint/run-ralph.sh
```

它会读取：

```bash
~/.config/ralph-codex/ralph.env
```

然后进入 `PROJECT_ROOT` 并执行：

```bash
$RALPH_DIR/ralph.sh --tool codex "$RALPH_ITERATIONS"
```

## 8. 用 tmux 长时间运行

```bash
tmux new -s ralph
cd ~/codex-ralph
./linux-mint/run-ralph.sh
```

让任务留在后台：

```text
Ctrl+B
D
```

重新进入：

```bash
tmux attach -t ralph
```

## 9. 用 systemd user service 长时间运行

安装服务：

```bash
cd ~/codex-ralph
./linux-mint/install-systemd-user.sh
```

启动：

```bash
systemctl --user start ralph-codex.service
```

查看状态：

```bash
systemctl --user status ralph-codex.service
```

查看日志：

```bash
journalctl --user -u ralph-codex.service -f
```

停止：

```bash
systemctl --user stop ralph-codex.service
```

如果希望退出登录后 user service 继续运行：

```bash
loginctl enable-linger "$USER"
```

## 10. 内网远程访问

如果你要在 Windows 上通过 SSH 查看运行状态，或部署后从内网浏览器访问 Web 页面，使用这两个脚本：

```bash
cd ~/codex-ralph
./linux-mint/setup-remote-access.sh --enable-ufw --allow-from 192.168.1.10
./linux-mint/remote-status.sh
```

把 `192.168.1.10` 改成 Windows 那台电脑的内网 IP。你的内网只有自己的两台电脑时，建议只允许 Windows IP，不需要放行整个网段。Web 页面建议优先用 SSH 隧道：

```powershell
ssh -L 5173:127.0.0.1:5173 user@linux-ip
```

Windows 浏览器打开：

```text
http://127.0.0.1:5173
```

完整方案见 [README.linux-mint-remote.zh-CN.md](README.linux-mint-remote.zh-CN.md)。

## 11. 运行产物

Ralph 会在 `RALPH_DIR` 下生成：

| 路径 | 说明 |
| --- | --- |
| `prd.json` | 当前 story 状态 |
| `progress.txt` | 跨轮记忆 |
| `.last-branch` | 上一次运行分支 |
| `.last-prd.json` | 上一次 PRD 快照 |
| `.ralph.lock` | 防重复运行锁 |
| `.ralph-state.json` | 最近一次状态 |
| `runs/` | 每轮结构化日志 |
| `archive/` | 分支切换归档 |

查看进度：

```bash
jq '.userStories[] | {id, title, passes}' ~/projects/my-app/scripts/ralph/prd.json
cat ~/projects/my-app/scripts/ralph/progress.txt
```

查看某轮日志：

```bash
ls ~/projects/my-app/scripts/ralph/runs
cat ~/projects/my-app/scripts/ralph/runs/<run-id>/iteration-1/status.json
cat ~/projects/my-app/scripts/ralph/runs/<run-id>/iteration-1/output.log
```

## 12. 常见问题

### codex 能运行，但请求失败

确认 Clash 端口：

```bash
bash -c ':</dev/tcp/127.0.0.1/7897' && echo ok
```

手动导出代理测试：

```bash
source ~/.config/ralph-codex/ralph.env
export http_proxy="$RALPH_HTTP_PROXY"
export https_proxy="$RALPH_HTTP_PROXY"
export all_proxy="$RALPH_ALL_PROXY"
codex exec --help
```

### git clone 或 npm install 不走代理

`run-ralph.sh` 启动时会导出代理变量。手动调试时也需要：

```bash
export http_proxy=http://127.0.0.1:7897
export https_proxy=http://127.0.0.1:7897
export all_proxy=socks5h://127.0.0.1:7897
```

### Docker 安装后仍提示权限不足

执行：

```bash
newgrp docker
```

或退出当前桌面/SSH 会话后重新登录。

### Ralph 提示 prd.json 不存在

确认 `prd.json` 放在 `RALPH_DIR`，不是项目根目录：

```bash
ls "$RALPH_DIR/prd.json"
```

### Ralph 提示已有进程运行

查看：

```bash
cat "$RALPH_DIR/.ralph.lock"
pgrep -af ralph.sh
```

如果确认没有进程，重新运行时 Ralph 会自动移除 stale lock。

### Codex 把 story 标完但没提交

runner 会自动把该 story 重置为 `passes:false`。这是预期行为，避免后续轮次跳过没有进入 git history 的工作。

## 13. 建议工作流

1. 运行 `./linux-mint/bootstrap.sh`。
2. 编辑 `~/.config/ralph-codex/ralph.env`。
3. 把 `ralph.sh`、`CODEX.md`、`prd.json` 放到目标项目的 `scripts/ralph/`。
4. 运行 `./linux-mint/check-env.sh`。
5. 如果要从 Windows 远程管理，运行 `./linux-mint/setup-remote-access.sh`。
6. 先用 tmux 跑 3 到 5 轮。
7. 检查 `progress.txt`、`runs/`、`git log`。
8. 稳定后改用 systemd user service 长时间运行。
