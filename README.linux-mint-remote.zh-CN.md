# Linux Mint 远程访问方案

这份文档用于你的内网方案：Windows 电脑通过 SSH 登录 Linux Mint 查看 Ralph 运行状态；部署后的 Web 页面可以通过内网地址直接访问，或者通过 SSH 隧道访问。

推荐拓扑：

```text
Windows PowerShell / Terminal
  ├─ ssh user@linux-ip             查看状态、日志、tmux、systemd
  ├─ http://linux-ip:5173          直接访问内网 Web 页面
  └─ ssh -L 5173:127.0.0.1:5173    更安全的本机端口转发
```

## 1. 安全边界

- 只在可信内网使用，不要在路由器上做公网端口转发。
- SSH 建议使用密钥登录，密码登录只适合初始配置。
- Web 端口只开放当前项目需要的端口，例如 `5173` 或 `3000`。
- 如果不需要让其他内网设备访问 Web 页面，优先用 SSH 隧道，不开放 Web 端口。
- Ralph 默认使用 Codex 的危险全权限模式，远程运行机器应当只放在可信网络里。

## 2. 在 Linux Mint 上启用 SSH

在 Linux 机器上执行：

```bash
cd ~/codex-ralph
chmod +x linux-mint/*.sh
./linux-mint/setup-remote-access.sh --enable-ufw --allow-from 192.168.1.10
```

把 `192.168.1.10` 改成 Windows 那台电脑的内网 IP。你现在只有自己的两台电脑，这种单 IP 放行比开放整个内网网段更合适。

如果以后有多台可信设备，也可以改成网段。常见网段示例：

```text
192.168.1.0/24
192.168.31.0/24
10.0.0.0/24
```

如果暂时不确定网段，可以先执行：

```bash
hostname -I
ip route | grep default
```

脚本会做这些事：

- 检查是否已经安装 `openssh-server`，缺失才安装。
- 启动并启用 `ssh` systemd 服务。
- 检查是否已经安装 `ufw`，缺失才安装。
- 放行 SSH 端口，默认 `22/tcp`。
- 如果设置了 `--allow-from`，只允许该 Windows IP 访问 SSH/Web 端口。
- 如果加了 `--enable-ufw`，启用防火墙。

如果你不想让脚本配置防火墙：

```bash
./linux-mint/setup-remote-access.sh --no-ufw
```

## 3. Windows 连接 Linux

在 Linux 上查看 IP：

```bash
hostname -I
```

假设 Linux IP 是 `192.168.1.23`，用户名是 `wkc`，Windows PowerShell 里执行：

```powershell
ssh wkc@192.168.1.23
```

如果你改了 SSH 端口，例如 `2222`：

```powershell
ssh -p 2222 wkc@192.168.1.23
```

## 4. 配置 SSH 密钥登录

在 Windows PowerShell 里生成密钥：

```powershell
ssh-keygen -t ed25519 -f $env:USERPROFILE\.ssh\linux-mint-ralph
```

把公钥追加到 Linux：

```powershell
type $env:USERPROFILE\.ssh\linux-mint-ralph.pub | ssh wkc@192.168.1.23 "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys && chmod 700 ~/.ssh && chmod 600 ~/.ssh/authorized_keys"
```

之后用密钥登录：

```powershell
ssh -i $env:USERPROFILE\.ssh\linux-mint-ralph wkc@192.168.1.23
```

可以在 Windows 的 `~/.ssh/config` 添加：

```sshconfig
Host mint-ralph
  HostName 192.168.1.23
  User wkc
  IdentityFile ~/.ssh/linux-mint-ralph
```

以后直接：

```powershell
ssh mint-ralph
```

## 5. 远程查看 Ralph 状态

一条命令查看整体状态：

```powershell
ssh mint-ralph "~/codex-ralph/linux-mint/remote-status.sh"
```

查看 systemd 服务：

```powershell
ssh mint-ralph "systemctl --user status ralph-codex.service --no-pager -l"
```

持续跟踪日志：

```powershell
ssh mint-ralph "journalctl --user -u ralph-codex.service -f"
```

查看 story 状态：

```powershell
ssh mint-ralph "jq '.userStories[] | {id,title,passes}' ~/projects/my-app/scripts/ralph/prd.json"
```

查看最近进度：

```powershell
ssh mint-ralph "tail -n 80 ~/projects/my-app/scripts/ralph/progress.txt"
```

如果用 `tmux` 运行：

```powershell
ssh -t mint-ralph "tmux attach -t ralph"
```

## 6. 内网直接访问 Web 页面

直接访问适合你希望 Windows 浏览器、手机、平板都能打开页面的场景。

先开放项目端口，例如 Vite 默认 `5173`：

```bash
cd ~/codex-ralph
./linux-mint/setup-remote-access.sh --web-port 5173 --enable-ufw --allow-from 192.168.1.10
```

Web 服务必须监听 `0.0.0.0`，不能只监听 `127.0.0.1`。

常见启动方式：

```bash
# Vite
npm run dev -- --host 0.0.0.0 --port 5173

# Next.js
npm run dev -- -H 0.0.0.0 -p 3000

# Python static server
python3 -m http.server 8000 --bind 0.0.0.0
```

然后在 Windows 浏览器打开：

```text
http://192.168.1.23:5173
```

## 7. 用 SSH 隧道访问 Web 页面

如果只是你自己在 Windows 上看页面，推荐 SSH 隧道。它不要求 Web 服务监听 `0.0.0.0`，也不需要开放 Web 防火墙端口。

Linux 上正常启动本地服务，例如：

```bash
npm run dev -- --host 127.0.0.1 --port 5173
```

Windows PowerShell 开一个隧道：

```powershell
ssh -L 5173:127.0.0.1:5173 mint-ralph
```

然后 Windows 浏览器打开：

```text
http://127.0.0.1:5173
```

Next.js 默认端口 `3000`：

```powershell
ssh -L 3000:127.0.0.1:3000 mint-ralph
```

浏览器打开：

```text
http://127.0.0.1:3000
```

## 8. 建议日常命令

Windows 上查看远程概况：

```powershell
ssh mint-ralph "~/codex-ralph/linux-mint/remote-status.sh"
```

启动 Ralph：

```powershell
ssh mint-ralph "systemctl --user start ralph-codex.service"
```

停止 Ralph：

```powershell
ssh mint-ralph "systemctl --user stop ralph-codex.service"
```

看日志：

```powershell
ssh mint-ralph "journalctl --user -u ralph-codex.service -n 200 --no-pager"
```

跟随日志：

```powershell
ssh mint-ralph "journalctl --user -u ralph-codex.service -f"
```

看正在监听的 Web 端口：

```powershell
ssh mint-ralph "ss -ltnp"
```

## 9. 排查

### Windows 连不上 SSH

在 Linux 上检查：

```bash
systemctl status ssh --no-pager -l
sudo ufw status verbose
hostname -I
```

在 Windows 上测试：

```powershell
ssh -v mint-ralph
```

### Windows 打不开 Web 页面

先在 Linux 本机测：

```bash
curl -I http://127.0.0.1:5173
ss -ltnp | grep 5173
```

如果只看到 `127.0.0.1:5173`，说明只能本机访问。要内网直接访问，需要让服务监听 `0.0.0.0`。

如果已经监听 `0.0.0.0:5173`，检查防火墙：

```bash
sudo ufw status verbose
```

### SSH 登录后 systemd user service 不运行

确认已经启用 linger：

```bash
loginctl show-user "$USER" -p Linger
```

如果是 `Linger=no`：

```bash
loginctl enable-linger "$USER"
```

### 机器 IP 变化

家用路由器可能会给 Linux 分配新的 IP。建议在路由器里给 Linux Mint 绑定 DHCP 静态租约，或者在 Windows 的 SSH config 里更新 `HostName`。
