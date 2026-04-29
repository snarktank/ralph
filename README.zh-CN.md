# Ralph Codex CLI 使用指南

Ralph 是一个面向 Codex CLI 的长时间自动任务循环框架。它把 `prd.json` 中的用户故事拆成多轮执行，每一轮都会启动一个新的 Codex CLI 实例，只处理一个最高优先级且尚未完成的 story。

默认工具是 Codex CLI；Amp 和 Claude Code 仍保留为兼容选项。

## 运行前准备

需要先安装并登录：

- Codex CLI
- Git
- Bash，Windows 推荐 Git Bash 或 WSL
- `jq`

Windows Git Bash 下如果 `codex` 命令无法直接运行，可以使用：

```bash
CODEX_BIN=codex.cmd ./ralph.sh
```

## 放到你的项目里

在目标项目根目录执行：

```bash
mkdir -p scripts/ralph
cp /path/to/codex-ralph/ralph.sh scripts/ralph/
cp /path/to/codex-ralph/CODEX.md scripts/ralph/
cp /path/to/codex-ralph/prd.json.example scripts/ralph/
chmod +x scripts/ralph/ralph.sh
```

如果还想保留旧工具支持，可以额外复制：

```bash
cp /path/to/codex-ralph/prompt.md scripts/ralph/
cp /path/to/codex-ralph/CLAUDE.md scripts/ralph/
```

## 准备 prd.json

Ralph 的任务入口是 `prd.json`。它必须和 `ralph.sh` 放在同一个目录中。

可以先参考：

```bash
cp scripts/ralph/prd.json.example scripts/ralph/prd.json
```

基本结构如下：

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

规则：

- `branchName` 是 Ralph 会切换或创建的功能分支。
- `priority` 越小越先执行。
- `passes:false` 表示未完成。
- 每个 story 应该足够小，能在一轮 Codex 中完成。

## 启动 Ralph

默认使用 Codex CLI：

```bash
./scripts/ralph/ralph.sh
```

指定最大迭代次数：

```bash
./scripts/ralph/ralph.sh 20
```

显式选择工具：

```bash
./scripts/ralph/ralph.sh --tool codex 20
./scripts/ralph/ralph.sh --tool amp 20
./scripts/ralph/ralph.sh --tool claude 20
```

Codex 默认以危险全权限模式运行：

```bash
codex exec --dangerously-bypass-approvals-and-sandbox --cd <project-root> --color never
```

这适合无人值守自动化，但只应在你信任的仓库和隔离环境中使用。

## 常用环境变量

```bash
CODEX_BIN=codex.cmd ./scripts/ralph/ralph.sh 20
```

| 变量 | 作用 |
| --- | --- |
| `CODEX_BIN` | Codex 可执行文件，例如 `codex` 或 `codex.cmd` |
| `RALPH_MODEL` | 传给 `codex exec --model` 的模型名 |
| `RALPH_PROFILE` | 传给 `codex exec --profile` 的配置 profile |
| `RALPH_CODEX_FLAGS` | 追加到 `codex exec` 的额外参数 |
| `RALPH_MAX_RETRIES` | CLI、网络、权限类失败的重试次数，默认 `2` |
| `RALPH_MAX_STORY_FAILURES` | 同一个 story 连续失败多少次后停止，默认 `3` |
| `RALPH_REQUIRE_CLEAN` | 设为 `1` 时，如果 git 工作区不干净则停止 |

示例：

```bash
CODEX_BIN=codex.cmd RALPH_MAX_RETRIES=3 RALPH_MAX_STORY_FAILURES=2 ./scripts/ralph/ralph.sh 20
```

## Ralph 每轮会做什么

每一轮会：

1. 读取 `prd.json`。
2. 读取 `progress.txt`。
3. 切换或创建 `branchName` 指定的分支。
4. 选择优先级最高且 `passes:false` 的 story。
5. 启动一个新的 Codex CLI 实例。
6. 让 Codex 实现该 story、运行检查、更新 `prd.json`、追加 `progress.txt`。
7. 要求 Codex 提交本轮变更。
8. 保存本轮日志。
9. 如果所有 story 都 `passes:true`，停止循环。

如果 Codex 把 story 标为完成但没有产生新 commit，Ralph 会把该 story 重置为 `passes:false`，避免跳过未提交的工作。

## 运行产物

Ralph 会生成这些文件，默认不提交：

| 文件或目录 | 说明 |
| --- | --- |
| `prd.json` | 当前任务列表 |
| `progress.txt` | 跨轮记忆和执行记录 |
| `.last-branch` | 上一次运行的 branch |
| `.last-prd.json` | 上一次运行的 PRD 快照，用于正确归档 |
| `.ralph.lock` | 防止同一目录重复运行 |
| `.ralph-state.json` | 最近一次迭代状态 |
| `runs/` | 每次运行的结构化日志 |
| `archive/` | branch 变化时归档旧运行 |

每一轮日志位于：

```text
runs/YYYYMMDD-HHMMSS/iteration-N/
```

常见文件：

```text
prompt.md
output.log
last-message.md
status.json
git-status.txt
diff-stat.txt
```

## 查看进度

查看 story 状态：

```bash
cat scripts/ralph/prd.json | jq '.userStories[] | {id, title, passes}'
```

查看跨轮记录：

```bash
cat scripts/ralph/progress.txt
```

查看最近提交：

```bash
git log --oneline -10
```

查看某一轮日志：

```bash
cat scripts/ralph/runs/<run-id>/iteration-1/status.json
cat scripts/ralph/runs/<run-id>/iteration-1/output.log
```

## 停止和恢复

停止 Ralph：

```bash
Ctrl+C
```

恢复时重新运行同一命令即可：

```bash
./scripts/ralph/ralph.sh 20
```

Ralph 会根据 `prd.json` 中的 `passes` 状态继续执行未完成 story。

如果异常退出后留下 `.ralph.lock`，下一次启动会检查里面的 PID。如果对应进程已经不存在，Ralph 会自动移除 stale lock。

## 写好 PRD 的建议

- 每个 story 只做一件事。
- 先后端和数据结构，再 UI。
- 每个 acceptance criterion 必须可验证。
- 每个 story 都包含 `Typecheck passes`。
- UI story 应包含浏览器验证要求。
- 不要把“做完整个功能”写成一个 story。

## 常见问题

### 找不到 jq

安装 `jq` 并确保它在 Bash 的 `PATH` 中。

Windows 可以通过包管理器或 Git Bash/WSL 环境安装。

### Windows 下 codex 无法执行

PowerShell 可能会被执行策略拦住 `codex.ps1`。在 Git Bash 中运行时可以指定：

```bash
CODEX_BIN=codex.cmd ./scripts/ralph/ralph.sh
```

### Codex 一直失败同一个 story

查看：

```bash
cat scripts/ralph/runs/<run-id>/iteration-N/output.log
cat scripts/ralph/runs/<run-id>/iteration-N/status.json
```

如果是 story 本身太大，把它拆成更小的 stories。

### 为什么需要 commit

Ralph 的长期记忆依赖 git history、`progress.txt` 和 `prd.json`。如果 story 被标记完成但没有 commit，下一轮无法可靠理解已经做了什么，所以 runner 会把它重置为未完成。
