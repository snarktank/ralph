# Ralph Linux 使用指南

这份文档面向 Linux 服务器或 Linux 开发机，说明如何用 Ralph 运行 Codex CLI 长时间自动任务循环。

Ralph 的核心流程是：读取 `prd.json`，每轮启动一个新的 Codex CLI 实例，只处理一个未完成 story，完成后更新 `prd.json`、追加 `progress.txt`、提交 git commit，然后继续下一轮。

## 1. 系统环境

推荐环境：

- Ubuntu 22.04/24.04、Debian 12、Fedora、Arch Linux 或同类发行版
- Bash
- Git
- `jq`
- Node.js/npm，如果你通过 npm 安装 Codex CLI
- 已登录并可正常运行的 Codex CLI

先确认基础命令：

```bash
bash --version
git --version
jq --version
codex --version
codex exec --help
```

## 2. 安装依赖

Debian/Ubuntu：

```bash
sudo apt update
sudo apt install -y git jq bash coreutils
```

Fedora：

```bash
sudo dnf install -y git jq bash coreutils
```

Arch Linux：

```bash
sudo pacman -S --needed git jq bash coreutils
```

如果系统没有 Node.js/npm，可以按你所在发行版的方式安装。示例：

```bash
node --version
npm --version
```

## 3. 安装和登录 Codex CLI

如果你使用 npm 安装 Codex CLI：

```bash
npm install -g @openai/codex
```

登录：

```bash
codex login
```

验证非交互模式可用：

```bash
codex exec --help
```

如果你的 Codex CLI 是通过其他方式安装的，只要 `codex exec --help` 能正常运行即可。

## 4. 把 Ralph 放入目标项目

假设你已经把本仓库克隆到 `/opt/codex-ralph`，目标项目在 `/srv/my-app`：

```bash
cd /srv/my-app
mkdir -p scripts/ralph
cp /opt/codex-ralph/ralph.sh scripts/ralph/
cp /opt/codex-ralph/CODEX.md scripts/ralph/
cp /opt/codex-ralph/prd.json.example scripts/ralph/
chmod +x scripts/ralph/ralph.sh
```

如果你还要保留旧工具兼容：

```bash
cp /opt/codex-ralph/prompt.md scripts/ralph/
cp /opt/codex-ralph/CLAUDE.md scripts/ralph/
```

## 5. 准备 prd.json

`prd.json` 必须和 `ralph.sh` 在同一个目录：

```bash
cd /srv/my-app
cp scripts/ralph/prd.json.example scripts/ralph/prd.json
```

编辑 `scripts/ralph/prd.json`：

```json
{
  "project": "MyApp",
  "branchName": "ralph/my-feature",
  "description": "Feature description",
  "userStories": [
    {
      "id": "US-001",
      "title": "Add database field",
      "description": "As a developer, I need to store the new field.",
      "acceptanceCriteria": [
        "Add database column",
        "Typecheck passes"
      ],
      "priority": 1,
      "passes": false,
      "notes": ""
    }
  ]
}
```

检查 JSON：

```bash
jq . scripts/ralph/prd.json >/dev/null
```

## 6. 启动 Ralph

在项目根目录运行：

```bash
cd /srv/my-app
./scripts/ralph/ralph.sh
```

指定最大迭代次数：

```bash
./scripts/ralph/ralph.sh 20
```

显式使用 Codex：

```bash
./scripts/ralph/ralph.sh --tool codex 20
```

Ralph 默认会调用：

```bash
codex exec --dangerously-bypass-approvals-and-sandbox --cd <project-root> --color never
```

这会绕过审批和沙箱，适合受控环境里的无人值守任务。不要在不可信仓库、不隔离的生产机器或含敏感凭据的目录中直接运行。

## 7. 常用环境变量

```bash
RALPH_MAX_RETRIES=3 RALPH_MAX_STORY_FAILURES=2 ./scripts/ralph/ralph.sh 20
```

| 变量 | 说明 |
| --- | --- |
| `CODEX_BIN` | Codex 可执行文件，默认 `codex` |
| `RALPH_MODEL` | 传给 `codex exec --model` |
| `RALPH_PROFILE` | 传给 `codex exec --profile` |
| `RALPH_CODEX_FLAGS` | 追加到 `codex exec` 的额外参数 |
| `RALPH_MAX_RETRIES` | CLI、网络、权限类失败重试次数，默认 `2` |
| `RALPH_MAX_STORY_FAILURES` | 同一 story 连续失败多少次后停止，默认 `3` |
| `RALPH_REQUIRE_CLEAN` | 设为 `1` 时，git 工作区不干净就停止 |

示例：

```bash
RALPH_MODEL=gpt-5.4 RALPH_MAX_RETRIES=3 ./scripts/ralph/ralph.sh 30
```

如果你安装的 Codex 命令不叫 `codex`：

```bash
CODEX_BIN=/usr/local/bin/codex ./scripts/ralph/ralph.sh 20
```

## 8. 长时间运行方式

### tmux 推荐

```bash
tmux new -s ralph
cd /srv/my-app
./scripts/ralph/ralph.sh 50
```

退出 tmux 会话但保持任务运行：

```text
Ctrl+B
D
```

重新进入：

```bash
tmux attach -t ralph
```

### nohup

```bash
cd /srv/my-app
nohup ./scripts/ralph/ralph.sh 50 > ralph.nohup.log 2>&1 &
echo $! > ralph.pid
```

查看输出：

```bash
tail -f ralph.nohup.log
```

停止：

```bash
kill "$(cat ralph.pid)"
```

### systemd 用户服务

创建目录：

```bash
mkdir -p ~/.config/systemd/user
```

创建 `~/.config/systemd/user/ralph-my-app.service`：

```ini
[Unit]
Description=Ralph Codex loop for my-app
After=network-online.target

[Service]
Type=simple
WorkingDirectory=/srv/my-app
Environment=RALPH_MAX_RETRIES=3
Environment=RALPH_MAX_STORY_FAILURES=3
ExecStart=/srv/my-app/scripts/ralph/ralph.sh 50
Restart=no

[Install]
WantedBy=default.target
```

启动：

```bash
systemctl --user daemon-reload
systemctl --user start ralph-my-app.service
```

查看日志：

```bash
journalctl --user -u ralph-my-app.service -f
```

如果需要退出 SSH 后仍保持 user service 运行：

```bash
loginctl enable-linger "$USER"
```

## 9. 查看进度和日志

查看 story 状态：

```bash
jq '.userStories[] | {id, title, passes}' scripts/ralph/prd.json
```

查看跨轮记忆：

```bash
cat scripts/ralph/progress.txt
```

查看最近提交：

```bash
git log --oneline -10
```

查看 Ralph 运行目录：

```bash
find scripts/ralph/runs -maxdepth 2 -type f | sort
```

查看某一轮：

```bash
cat scripts/ralph/runs/<run-id>/iteration-1/status.json
cat scripts/ralph/runs/<run-id>/iteration-1/output.log
cat scripts/ralph/runs/<run-id>/iteration-1/git-status.txt
cat scripts/ralph/runs/<run-id>/iteration-1/diff-stat.txt
```

## 10. 停止和恢复

手动停止：

```text
Ctrl+C
```

或 kill 对应进程：

```bash
pgrep -af ralph.sh
kill <pid>
```

恢复：

```bash
cd /srv/my-app
./scripts/ralph/ralph.sh 20
```

Ralph 会根据 `prd.json` 中的 `passes` 状态继续处理未完成 story。

如果异常退出留下 `.ralph.lock`，下次启动会检查里面的 PID。若进程已不存在，Ralph 会自动删除 stale lock。

## 11. 归档和运行产物

Ralph 会生成：

| 路径 | 说明 |
| --- | --- |
| `scripts/ralph/prd.json` | 当前任务列表 |
| `scripts/ralph/progress.txt` | 跨轮记忆 |
| `scripts/ralph/.last-branch` | 上一次 branch |
| `scripts/ralph/.last-prd.json` | 上一次 PRD 快照 |
| `scripts/ralph/.ralph.lock` | 运行锁 |
| `scripts/ralph/.ralph-state.json` | 最近一次状态 |
| `scripts/ralph/runs/` | 结构化运行日志 |
| `scripts/ralph/archive/` | branch 切换时的旧运行归档 |

这些文件默认应被 `.gitignore` 忽略，不建议提交。

## 12. 常见问题

### codex 命令找不到

检查 PATH：

```bash
which codex
codex --version
```

如果 Codex 安装在自定义路径：

```bash
CODEX_BIN=/custom/path/codex ./scripts/ralph/ralph.sh
```

### jq 命令找不到

安装：

```bash
sudo apt install -y jq
```

或按你的发行版使用 `dnf`、`pacman` 安装。

### 权限不足

确认脚本可执行：

```bash
chmod +x scripts/ralph/ralph.sh
```

确认项目目录可写：

```bash
touch .ralph-write-test && rm .ralph-write-test
```

### 同一个 story 连续失败

查看失败日志：

```bash
cat scripts/ralph/runs/<run-id>/iteration-N/output.log
cat scripts/ralph/runs/<run-id>/iteration-N/status.json
```

常见原因：

- story 太大，需要拆分。
- acceptance criteria 不可验证。
- 项目本身缺少测试或 typecheck 命令。
- Codex 无法访问必要环境变量或服务。

### story 被标记完成但没有提交

Ralph 会自动把该 story 重置为 `passes:false`。这是为了避免跳过未提交的工作，因为后续迭代主要依赖 git history、`progress.txt` 和 `prd.json` 作为长期记忆。

## 13. 建议的 Linux 运行习惯

- 在隔离的开发机、容器或临时 VM 中运行。
- 先用 1 到 2 个小 story 做试跑。
- 每个 story 保持小而明确。
- 保持项目的 typecheck、lint、test 命令可用。
- 使用 tmux 或 systemd 管理长时间任务。
- 定期查看 `runs/` 和 `progress.txt`，不要只看最终输出。
