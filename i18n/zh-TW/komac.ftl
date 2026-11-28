### Komac 的繁體中文翻譯，譯自 i18n/en-US/komac.ftl。
###
### 術語採用台灣慣用詞及 Microsoft 繁體中文介面（如 winget）的譯法：pull request 譯為「提取要求」，
### fork 譯為「分叉」，upstream 譯為「上游」，token 譯為「權杖」，manifest 譯為「資訊清單」，
### package 譯為「套件」，repository 譯為「存放庫」。

## 資訊清單欄位

field-author = 作者
field-channel = 通道
field-command = 命令
    .plural = 命令
    .help = 用於執行此套件的命令或別名清單
field-copyright = 著作權
    .help = 範例：Copyright (c) Microsoft Corporation
field-copyright-url = 著作權 URL
field-custom-switch = 自訂參數
    .help = 範例：/norestart, -norestart
field-description = 描述
field-file-extension = 副檔名
    .plural = 副檔名
    .help = 此套件可支援的副檔名清單
field-installation-notes = 安裝注意事項
field-installer-return-code = 安裝程式傳回碼
    .plural = 安裝程式傳回碼
    .help = 除了 winget 已知的預設值之外，表示安裝成功的其他非零結束代碼清單
field-install-modes = 安裝模式
field-language-tag = 語言標記
    .help = 範例：en-US
field-license = 授權
    .help = 範例：MIT, GPL-3.0, Freeware, Proprietary
field-license-url = 授權 URL
field-moniker = 簡稱
    .help = 範例：vscode
field-package-identifier = 套件識別碼
    .help = 套件識別碼的格式為 Publisher.package
    .placeholder = Publisher.package
field-package-name = 套件名稱
    .help = 範例：Microsoft Teams
field-package-url = 套件 URL
field-package-version = 套件版本
    .help = 範例：1.2.3
field-portable-command-alias = 可攜式命令別名
field-protocol = 通訊協定
    .plural = 通訊協定
    .help = 此套件可處理的通訊協定清單。範例：http, https
field-publisher = 發行者
    .help = 範例：Microsoft Corporation
field-publisher-support-url = 發行者支援 URL
field-publisher-url = 發行者 URL
field-release-notes-url = 版本資訊 URL
field-short-description = 簡短描述
field-silent-switch = 靜默安裝參數
    .help = 範例：/S, -verysilent, /qn, --silent, /exenoui
field-silent-with-progress-switch = 顯示進度的靜默安裝參數
    .help = 範例：/S, -silent, /qb, /exebasicui
field-tag = 標籤
    .help = 範例：zip, c++, photos, OBS
field-upgrade-behavior = 升級行為

## 提示

prompt-too-many-items = 最多允許 { $max } 項，但提供了 { $count } 項
prompt-installer-url = 第 { $position } 個安裝程式 URL
    .invalid = 請輸入有效的 URL
    .help = 如果沒有更多 URL，請按 ESC
prompt-portable-exe = { $file } 是可攜式 exe 嗎？
prompt-nested-files = 選擇巢狀檔案
prompt-nested-portable-command-alias = { $path } 的可攜式命令別名：
prompt-removal-reason = 請說明移除 { $identifier } 版本 { $version } 的原因
prompt-remove-version = 是否建立提取要求以移除 { $identifier } { $version }？
prompt-remove-dead-version = { $identifier } { $version } 的所有 InstallerUrls 都傳回了 { $status }。是否移除？
prompt-remove-anyway = 仍要移除嗎？
prompt-remove-token = 是否移除目前儲存的權杖？
prompt-proceed = 是否繼續？
prompt-token = 輸入 GitHub 權杖
prompt-new-token = 請輸入要設定的新權杖
prompt-select-branches = 請選擇要刪除的分支
prompt-select-packages = 請選擇要提交的套件
prompt-submit-option = 要如何處理 { $identifier } { $version }？
    .submit = 提交
    .edit = 編輯
    .exit = 結束

## 終端機輸出

success = 成功
latest-version = { $identifier } 的最新版本：{ $version }
manifests-written = 已{ $success }將所有資訊清單檔案寫入 { $path }
pull-request-exists = { $identifier } { $version } 已有一個{ $state ->
        [open] 開啟中的
        [merged] 已合併的
       *[closed] 已關閉的
    }提取要求，建立於 { $date } { $time }
pull-request-closed-proceeding = 現有的提取要求已關閉且未合併，因此繼續
pull-request-creating = 正在為 { $identifier } { $version } 建立提取要求
pull-request-creating-removal = 正在建立提取要求以移除 { $identifier } { $version }
pull-request-created = 已{ $success }建立 { $pull_request }
pull-request-rate-limit = 上一個提取要求建立於 { $elapsed } 前。等待 { $remaining }
downloading = 正在下載 { $file }
version-count = 共有 { $count } 個版本
no-packages-to-submit = 在 { $path } 中找不到可提交的有效套件
update-available = { $name } 有新版本可用：

## cleanup

cleanup-retrieving = 正在取得關聯了{ $state ->
        [merged] 已合併
        [closed] 已關閉
       *[any] 已合併或已關閉
    }提取要求的分支
cleanup-no-branches = 沒有關聯了{ $state ->
        [merged] 已合併
        [closed] 已關閉
       *[any] 已合併或已關閉
    }提取要求且可以刪除的分支
cleanup-none-deleted = 未刪除任何分支
cleanup-deleting = 正在刪除所選的 { $count } 個分支
cleanup-deleted = 已{ $success }刪除所選的 { $count } 個分支

## remove 和 remove-dead-versions

remove-version-warning = 僅應在必要時移除套件
remove-dead-checking = 正在檢查 { $identifier } { $version }
remove-dead-pull-request-exists = { $identifier } { $version } 的所有 InstallerUrls 都傳回了 { $status }，但此版本已有一個{ $state ->
        [open] 開啟中的
        [merged] 已合併的
       *[closed] 已關閉的
    }提取要求，建立於 { $date } { $time }。

## sync

commits = { $count } 個提交
merge-summary = 從 { $upstream } 到 { $fork } 的 { $commits }
sync-up-to-date = { $fork } 已與 { $upstream } { $status }
    .status = 保持同步
sync-fork-ahead = { $fork } 並未落後於 { $upstream }，因此無需合併。它有 { $commits } 是 { $upstream_name } 所沒有的
sync-merged = 已{ $success }合併{ $summary }（{ $kind ->
        [merge] 合併
       *[fast-forward] 快轉
    }）
sync-merging = 正在合併{ $summary }

## token

token-none-stored = 平台的安全儲存區中目前沒有儲存權杖
token-deleted = 已{ $success }從平台的安全儲存區中刪除儲存的權杖
token-not-deleted = 未刪除任何權杖
token-stored = 已{ $success }將權杖儲存到平台的安全儲存區中

## 錯誤

error-no-token-in-ci = 未提供或儲存權杖。請透過 `GITHUB_TOKEN` 環境變數或 `--token` 提供權杖。
error-invalid-token = GitHub 權杖無效。
error-failed-to-connect = 無法連線到 GitHub。請檢查網路連線。
error-invalid-repository = GitHub 存放庫的格式應為 `owner/name`，但實際為 `{ $value }`
error-package-not-found = { $identifier } 不存在於 { $upstream } 中
error-manifest-not-found = 在 { $path } 中找不到 { $manifest_type } 資訊清單
error-no-valid-files = 找不到 { $path } 的有效檔案
error-version-not-found = { $identifier } 版本 { $version } 不存在於 { $upstream } 中
error-replacement-version-not-found = 取代版本 { $version } 不存在於 { $upstream } 中。最接近的版本是 { $closest }
error-pull-request-exists = { $identifier } { $version } 已存在一個{ $state ->
        [open] 開啟中的
        [merged] 已合併的
       *[closed] 已關閉的
    }提取要求。使用 --skip-pr-check 仍可提交
error-download-failed = { $url } 傳回了 { $status }
error-unexpected-content-type = { $file } 的內容類型為 { $content_types }，但預期為 { $application } 或 { $octet_stream } 內容類型
list-and = { $items } 和 { $last }
error-unknown-shell = 無法從環境中判斷目前的 shell
    .suggestion = 請明確指定 shell
error-sync-upstream-not-found = 找不到 { $upstream }，或權杖無法存取它
error-sync-fork-not-found = 找不到 { $fork }
    .suggestion = 請先在 GitHub 上分叉 { $upstream }，或使用 --fork 指定你的分叉
error-sync-not-a-fork = { $fork } 不是分叉，因此無法與 { $upstream } 同步
error-sync-fork-of-other = { $fork } 是 { $parent } 的分叉，而不是 { $upstream } 的分叉。GitHub 只能從父存放庫同步分叉
error-sync-parent-inaccessible = { $fork } 是一個分叉，但權杖無法存取其父存放庫，因此無法確認它是否為 { $upstream } 的分叉
error-sync-merge = 無法合併{ $summary }

## 資訊清單編輯器

editor-help = 按 { $quit } 結束，{ $save } 儲存，{ $search } 搜尋，{ $switch } 切換緩衝區
editor-search-help = 按 { $enter } 跳至第一個相符項目並關閉，{ $close } 關閉，{ $next } 搜尋下一個，{ $previous } 搜尋上一個
editor-search = 搜尋
editor-search-error = 搜尋：{ $error }
editor-pattern-not-found = 找不到相符項目
editor-switched-buffer = 已切換到緩衝區 #{ $number }
editor-saved = 已儲存！
editor-modified = [已修改]

## 命令列說明

command-komac = 適用於 winget-pkgs 的資訊清單建立工具
command-analyze = 分析檔案並輸出其相關資訊
command-cleanup = 在上游存放庫的分叉中，尋找向上游存放庫發出的提取要求已合併或已關閉的分支，並提示選擇要刪除的分支
command-complete = 為指定的 shell 輸出自動完成指令碼。使用範例：
    .long =
        為指定的 shell 輸出自動完成指令碼。使用範例：

        Bash: echo "source <(komac complete bash)" >> ~/.bashrc
        Elvish: echo "eval (komac complete elvish | slurp)" >> ~/.elvish/rc.elv
        Fish: echo "source (komac complete fish | psub)" >> ~/.config/fish/config.fish
        PowerShell: echo "komac complete powershell | Out-String | Invoke-Expression" >> $PROFILE
        Zsh: echo "source <(komac complete zsh)" >> ~/.zshrc
command-list = 列出指定套件的所有版本
command-new = 從頭建立新的套件
command-remove = 從上游存放庫（預設為 microsoft/winget-pkgs）中移除一個版本
    .long =
        從上游存放庫（預設為 microsoft/winget-pkgs）中移除一個版本

        若要移除一個套件，必須移除該套件的所有版本
command-show = 輸出指定套件和版本的資訊清單
command-sync = 將上游存放庫（預設為 microsoft/winget-pkgs）的變更合併到分叉中
command-token-remove = 移除儲存的權杖
command-token-update = 更新儲存的權杖
command-update = 為現有的套件新增一個版本

arg-analyze-no-hash = 略過計算檔案的雜湊值
arg-analyze-hash = 計算檔案的雜湊值並將其寫入 `InstallerSha256` 欄位
arg-cleanup-all = 自動刪除所有相關分支
arg-cleanup-only-closed = 僅刪除已關閉的分支
arg-cleanup-only-merged = 僅刪除已合併的分支
arg-complete-shell = 指定要為其產生自動完成指令碼的 shell
    .long =
        指定要為其產生自動完成指令碼的 shell。

        如果未指定，將根據目前的環境推斷 shell。
arg-concurrent-downloads = 同時下載的安裝程式數量
arg-created-with = 呼叫 Komac 的外部工具名稱
arg-created-with-url = 呼叫 Komac 的外部工具 URL
arg-dry-run = 執行但不提交
arg-exit = 自動略過提示並結束（如果指定了 --output，則會儲存資訊清單）
arg-fast = 使用每分鐘速率限制，可能會在 7.5 分鐘內達到每小時速率限制
arg-identifier = 套件的唯一識別碼
arg-list-count = 輸出該套件的版本數量
arg-list-json = 以 JSON 格式輸出版本
arg-list-pretty-json = 以格式化的 JSON 輸出版本
arg-list-yaml = 以 YAML 格式輸出版本
arg-new-dry-run = 執行但不提示也不提交
arg-new-resolves = 新增此套件或版本將解決的問題清單
arg-open-pr = 自動開啟提取要求連結
arg-output = 資訊清單的輸出目錄
arg-package-identifier = 套件的唯一識別碼
arg-package-version = 套件的版本
arg-remove-dead-versions-after = 檢查高於指定版本的版本
arg-remove-dead-versions-auto = 自動建立提取要求以移除失效版本，不進行提示
arg-remove-dead-versions-before = 檢查低於指定版本的版本
arg-remove-dead-versions-concurrent = 同時檢查的版本數量
arg-remove-no-warning = 不顯示套件移除警告
arg-remove-resolves = 移除此版本將解決的問題清單
arg-repository = 讀取資訊清單以及發出提取要求的 GitHub 存放庫
    .long =
        讀取資訊清單以及發出提取要求的 GitHub 存放庫

        使用此選項可改用第三方或私人存放庫，而不是 microsoft/winget-pkgs。你的分叉必須與此存放庫同名，除非命令允許你指定分叉，例如 `sync --fork`。
arg-show-default-locale-manifest = 切換為顯示預設地區設定資訊清單
arg-show-installer-manifest = 切換為顯示安裝程式資訊清單
arg-show-locale-manifests = 切換為顯示所有地區設定資訊清單
arg-show-version-manifest = 切換為顯示版本資訊清單
arg-skip-pr-check = 略過檢查現有的提取要求
arg-submit = 自動提交提取要求
arg-submit-resolves = 提交此版本將解決的問題清單
arg-submit-skip-prompt = 略過提交套件的確認提示
arg-submit-submit-all = 提交資料夾中的所有套件
arg-sync-branch = 要同步的分支。未指定時，分叉的預設分支會與上游的預設分支同步。指定時，該分支會與上游的同名分支同步
arg-sync-fork = 要同步的分叉（如果不是 `<you>/<upstream name>`），例如重新命名過的分叉或組織擁有的分叉
arg-sync-fork-owner = 分叉的擁有者（如果不是已驗證的使用者）。指定 --fork 時將忽略此選項
arg-token = 具有 `public_repo` 範圍的 GitHub 個人存取權杖
arg-token-remove-skip-prompt = 略過刪除權杖的確認提示
arg-token-update-token = 要儲存的新權杖
arg-update-release-notes-url = 套件版本資訊的 URL
arg-update-replace = 要取代的套件版本
arg-update-resolves = 更新此套件將解決的問題清單
arg-urls = 套件安裝程式的直接下載 URL，以空格分隔
arg-version = 套件的版本
