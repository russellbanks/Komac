### Komac 的简体中文翻译，译自 i18n/en-US/komac.ftl。
###
### 术语与 GitHub 中文界面保持一致：pull request 译为“拉取请求”，fork 译为“复刻”，
### upstream 译为“上游”，token 译为“令牌”，manifest 译为“清单”。

## 清单字段

field-author = 作者
field-channel = 渠道
field-command = 命令
    .plural = 命令
    .help = 用于运行该软件包的命令或别名列表
field-copyright = 版权
    .help = 示例：Copyright (c) Microsoft Corporation
field-copyright-url = 版权 URL
field-custom-switch = 自定义开关
    .help = 示例：/norestart, -norestart
field-description = 描述
field-file-extension = 文件扩展名
    .plural = 文件扩展名
    .help = 该软件包可支持的文件扩展名列表
field-installation-notes = 安装说明
field-installer-return-code = 安装程序返回码
    .plural = 安装程序返回码
    .help = 除 winget 已知的默认值之外，表示安装成功的其他非零退出码列表
field-install-modes = 安装模式
field-language-tag = 语言标记
    .help = 示例：en-US
field-license = 许可证
    .help = 示例：MIT, GPL-3.0, Freeware, Proprietary
field-license-url = 许可证 URL
field-moniker = 简称
    .help = 示例：vscode
field-package-identifier = 软件包标识符
    .help = 软件包标识符的格式为 Publisher.package
    .placeholder = Publisher.package
field-package-name = 软件包名称
    .help = 示例：Microsoft Teams
field-package-url = 软件包 URL
field-package-version = 软件包版本
    .help = 示例：1.2.3
field-portable-command-alias = 便携版命令别名
field-protocol = 协议
    .plural = 协议
    .help = 该软件包可处理的协议列表。示例：http, https
field-publisher = 发布者
    .help = 示例：Microsoft Corporation
field-publisher-support-url = 发布者支持 URL
field-publisher-url = 发布者 URL
field-release-notes-url = 发行说明 URL
field-short-description = 简短描述
field-silent-switch = 静默安装开关
    .help = 示例：/S, -verysilent, /qn, --silent, /exenoui
field-silent-with-progress-switch = 显示进度的静默安装开关
    .help = 示例：/S, -silent, /qb, /exebasicui
field-tag = 标签
    .help = 示例：zip, c++, photos, OBS
field-upgrade-behavior = 升级行为

## 提示

prompt-too-many-items = 最多允许 { $max } 项，但提供了 { $count } 项
prompt-installer-url = 第 { $position } 个安装程序 URL
    .invalid = 请输入有效的 URL
    .help = 如果没有更多 URL，请按 ESC
prompt-portable-exe = { $file } 是便携版 exe 吗？
prompt-nested-files = 选择嵌套文件
prompt-nested-portable-command-alias = { $path } 的便携版命令别名：
prompt-removal-reason = 请说明移除 { $identifier } 版本 { $version } 的原因
prompt-remove-version = 是否创建拉取请求以移除 { $identifier } { $version }？
prompt-remove-dead-version = { $identifier } { $version } 的所有 InstallerUrls 都返回了 { $status }。是否移除？
prompt-remove-anyway = 仍要移除吗？
prompt-remove-token = 是否移除当前存储的令牌？
prompt-proceed = 是否继续？
prompt-token = 输入 GitHub 令牌
prompt-new-token = 请输入要设置的新令牌
prompt-select-branches = 请选择要删除的分支
prompt-select-packages = 请选择要提交的软件包
prompt-submit-option = 要如何处理 { $identifier } { $version }？
    .submit = 提交
    .edit = 编辑
    .exit = 退出

## 终端输出

success = 成功
latest-version = { $identifier } 的最新版本：{ $version }
manifests-written = 已{ $success }将所有清单文件写入 { $path }
pull-request-exists = { $identifier } { $version } 已有一个{ $state ->
        [open] 打开的
        [merged] 已合并的
       *[closed] 已关闭的
    }拉取请求，创建于 { $date } { $time }
pull-request-closed-proceeding = 现有拉取请求已关闭且未合并，因此继续
pull-request-creating = 正在为 { $identifier } { $version } 创建拉取请求
pull-request-creating-removal = 正在创建拉取请求以移除 { $identifier } { $version }
pull-request-created = 已{ $success }创建 { $pull_request }
pull-request-rate-limit = 上一个拉取请求创建于 { $elapsed } 前。等待 { $remaining }
downloading = 正在下载 { $file }
version-count = 共有 { $count } 个版本
no-packages-to-submit = 在 { $path } 中未找到可提交的有效软件包
update-available = { $name } 有新版本可用：

## cleanup

cleanup-retrieving = 正在获取关联了{ $state ->
        [merged] 已合并
        [closed] 已关闭
       *[any] 已合并或已关闭
    }拉取请求的分支
cleanup-no-branches = 没有关联了{ $state ->
        [merged] 已合并
        [closed] 已关闭
       *[any] 已合并或已关闭
    }拉取请求且可以删除的分支
cleanup-none-deleted = 未删除任何分支
cleanup-deleting = 正在删除所选的 { $count } 个分支
cleanup-deleted = 已{ $success }删除所选的 { $count } 个分支

## remove 和 remove-dead-versions

remove-version-warning = 仅应在必要时移除软件包
remove-dead-checking = 正在检查 { $identifier } { $version }
remove-dead-pull-request-exists = { $identifier } { $version } 的所有 InstallerUrls 都返回了 { $status }，但此版本已有一个{ $state ->
        [open] 打开的
        [merged] 已合并的
       *[closed] 已关闭的
    }拉取请求，创建于 { $date } { $time }。

## sync

commits = { $count } 个提交
merge-summary = 从 { $upstream } 到 { $fork } 的 { $commits }
sync-up-to-date = { $fork } 已与 { $upstream } { $status }
    .status = 保持同步
sync-fork-ahead = { $fork } 并未落后于 { $upstream }，因此无需合并。它有 { $commits } 是 { $upstream_name } 所没有的
sync-merged = 已{ $success }合并{ $summary }（{ $kind ->
        [merge] 合并
       *[fast-forward] 快进
    }）
sync-merging = 正在合并{ $summary }

## token

token-none-stored = 平台的安全存储中当前没有存储令牌
token-deleted = 已{ $success }从平台的安全存储中删除存储的令牌
token-not-deleted = 未删除任何令牌
token-stored = 已{ $success }将令牌存储到平台的安全存储中

## 错误

error-no-token-in-ci = 未提供或存储令牌。请通过 `GITHUB_TOKEN` 环境变量或 `--token` 提供令牌。
error-invalid-token = GitHub 令牌无效。
error-failed-to-connect = 无法连接到 GitHub。请检查网络连接。
error-invalid-repository = GitHub 仓库的格式应为 `owner/name`，但实际为 `{ $value }`
error-package-not-found = { $identifier } 在 { $upstream } 中不存在
error-manifest-not-found = 在 { $path } 中未找到 { $manifest_type } 清单
error-no-valid-files = 未找到 { $path } 的有效文件
error-version-not-found = { $identifier } 版本 { $version } 在 { $upstream } 中不存在
error-replacement-version-not-found = 替换版本 { $version } 在 { $upstream } 中不存在。最接近的版本是 { $closest }
error-pull-request-exists = { $identifier } { $version } 已存在一个{ $state ->
        [open] 打开的
        [merged] 已合并的
       *[closed] 已关闭的
    }拉取请求。使用 --skip-pr-check 仍可提交
error-download-failed = { $url } 返回了 { $status }
error-unexpected-content-type = { $file } 的内容类型为 { $content_types }，但预期为 { $application } 或 { $octet_stream } 内容类型
list-and = { $items } 和 { $last }
error-unknown-shell = 无法从环境中确定当前 shell
    .suggestion = 请明确指定 shell
error-sync-upstream-not-found = 未找到 { $upstream }，或令牌无法访问它
error-sync-fork-not-found = 未找到 { $fork }
    .suggestion = 请先在 GitHub 上复刻 { $upstream }，或使用 --fork 指定你的复刻
error-sync-not-a-fork = { $fork } 不是复刻，因此无法与 { $upstream } 同步
error-sync-fork-of-other = { $fork } 是 { $parent } 的复刻，而不是 { $upstream } 的复刻。GitHub 只能从父仓库同步复刻
error-sync-parent-inaccessible = { $fork } 是一个复刻，但令牌无法访问其父仓库，因此无法确认它是否为 { $upstream } 的复刻
error-sync-merge = 无法合并{ $summary }

## 清单编辑器

editor-help = 按 { $quit } 退出，{ $save } 保存，{ $search } 搜索，{ $switch } 切换缓冲区
editor-search-help = 按 { $enter } 跳转到第一个匹配项并关闭，{ $close } 关闭，{ $next } 搜索下一个，{ $previous } 搜索上一个
editor-search = 搜索
editor-search-error = 搜索：{ $error }
editor-pattern-not-found = 未找到匹配项
editor-switched-buffer = 已切换到缓冲区 #{ $number }
editor-saved = 已保存！
editor-modified = [已修改]

## 命令行帮助

command-komac = 用于 winget-pkgs 的清单创建工具
command-analyze = 分析文件并输出其相关信息
command-cleanup = 查找上游仓库的复刻中向上游仓库发起的拉取请求已合并或已关闭的分支，并提示选择要删除的分支
command-complete = 为指定的 shell 输出自动补全脚本。用法示例：
    .long =
        为指定的 shell 输出自动补全脚本。用法示例：

        Bash: echo "source <(komac complete bash)" >> ~/.bashrc
        Elvish: echo "eval (komac complete elvish | slurp)" >> ~/.elvish/rc.elv
        Fish: echo "source (komac complete fish | psub)" >> ~/.config/fish/config.fish
        PowerShell: echo "komac complete powershell | Out-String | Invoke-Expression" >> $PROFILE
        Zsh: echo "source <(komac complete zsh)" >> ~/.zshrc
command-list = 列出指定软件包的所有版本
command-new = 从头创建新的软件包
command-remove = 从上游仓库（默认为 microsoft/winget-pkgs）中移除一个版本
    .long =
        从上游仓库（默认为 microsoft/winget-pkgs）中移除一个版本

        要移除一个软件包，必须移除该软件包的所有版本
command-show = 输出指定软件包和版本的清单
command-sync = 将上游仓库（默认为 microsoft/winget-pkgs）的更改合并到复刻仓库中
command-token-remove = 移除存储的令牌
command-token-update = 更新存储的令牌
command-update = 为已有的软件包添加一个版本

arg-analyze-no-hash = 跳过计算文件的哈希值
arg-analyze-hash = 计算文件的哈希值并将其写入 `InstallerSha256` 字段
arg-cleanup-all = 自动删除所有相关分支
arg-cleanup-only-closed = 仅删除已关闭的分支
arg-cleanup-only-merged = 仅删除已合并的分支
arg-complete-shell = 指定要为其生成补全脚本的 shell
    .long =
        指定要为其生成补全脚本的 shell。

        如果未指定，将根据当前环境推断 shell。
arg-concurrent-downloads = 同时下载的安装程序数量
arg-created-with = 调用 Komac 的外部工具名称
arg-created-with-url = 调用 Komac 的外部工具 URL
arg-dry-run = 运行但不提交
arg-exit = 自动跳过提示并退出（如果指定了 --output，则会保存清单）
arg-fast = 使用每分钟速率限制，可能会在 7.5 分钟内达到每小时速率限制
arg-identifier = 软件包的唯一标识符
arg-list-count = 输出该软件包的版本数量
arg-list-json = 以 JSON 格式输出版本
arg-list-pretty-json = 以格式化的 JSON 输出版本
arg-list-yaml = 以 YAML 格式输出版本
arg-new-dry-run = 运行但不提示也不提交
arg-new-resolves = 添加此软件包或版本将解决的议题列表
arg-open-pr = 自动打开拉取请求链接
arg-output = 清单的输出目录
arg-package-identifier = 软件包的唯一标识符
arg-package-version = 软件包的版本
arg-remove-dead-versions-after = 检查高于指定版本的版本
arg-remove-dead-versions-auto = 自动创建拉取请求以移除失效版本，不进行提示
arg-remove-dead-versions-before = 检查低于指定版本的版本
arg-remove-dead-versions-concurrent = 同时检查的版本数量
arg-remove-no-warning = 不显示软件包移除警告
arg-remove-resolves = 移除此版本将解决的议题列表
arg-repository = 读取清单以及发起拉取请求的 GitHub 仓库
    .long =
        读取清单以及发起拉取请求的 GitHub 仓库

        使用此选项可改用第三方或私有仓库，而不是 microsoft/winget-pkgs。你的复刻必须与此仓库同名，除非命令允许你指定复刻，例如 `sync --fork`。
arg-show-default-locale-manifest = 切换为显示默认区域设置清单
arg-show-installer-manifest = 切换为显示安装程序清单
arg-show-locale-manifests = 切换为显示所有区域设置清单
arg-show-version-manifest = 切换为显示版本清单
arg-skip-pr-check = 跳过检查现有的拉取请求
arg-submit = 自动提交拉取请求
arg-submit-resolves = 提交此版本将解决的议题列表
arg-submit-skip-prompt = 跳过提交软件包的确认提示
arg-submit-submit-all = 提交文件夹中的所有软件包
arg-sync-branch = 要同步的分支。不指定时，复刻的默认分支会与上游的默认分支同步。指定时，该分支会与上游的同名分支同步
arg-sync-fork = 要同步的复刻（如果不是 `<you>/<upstream name>`），例如重命名过的复刻或组织拥有的复刻
arg-sync-fork-owner = 复刻的所有者（如果不是已认证的用户）。指定 --fork 时将忽略此选项
arg-token = 具有 `public_repo` 范围的 GitHub 个人访问令牌
arg-token-remove-skip-prompt = 跳过删除令牌的确认提示
arg-token-update-token = 要存储的新令牌
arg-update-release-notes-url = 软件包发行说明的 URL
arg-update-replace = 要替换的软件包版本
arg-update-resolves = 更新此软件包将解决的议题列表
arg-urls = 软件包安装程序的直接下载 URL，以空格分隔
arg-version = 软件包的版本
