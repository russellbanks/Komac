## 贡献

贡献使开源社区成为一个令人惊叹的学习、启发和创造的地方。**非常感谢**你所做的任何贡献。

* 如果你有添加或删除项目的建议，请随时 [打开一个问题(源仓库)](https://github.com/russellbanks/Komac/issues/new) 进行讨论，或者在编辑 *README.md* 文件后直接创建一个拉取请求进行必要的更改。
* 请确保检查你的拼写和语法。
* 为每个建议创建单独的 PR。
* 在发布你的第一个想法之前，请务必阅读 [行为准则](./CODE_OF_CONDUCT.md)。

### 创建拉取请求

1. Fork 项目
2. 创建你的功能分支 (`git checkout -b feat/new-feature`)
3. 提交你的更改 (`git commit -m 'Add some feature'`)
4. 推送到分支 (`git push origin feat/new-feature`)
5. 打开一个拉取请求

### 测试你的更改

使用 Docker 是测试代码的最简单方法。

> [!注意]
> 在 Windows 上使用 Docker 容器时，WSL 引擎不支持默认的密钥或令牌集合。这意味着在容器内测试时，即使使用 `komac token update`，GitHub 令牌也不会被存储。
> 
> 这是 keyring crate 中的 [已知问题](https://github.com/hwchen/keyring-rs/blob/47c8daf3e6178a2282ae3e8670d1ea7fa736b8cb/src/secret_service.rs#L73-L77)。
>
> 作为解决方法，你可以在容器内、`docker run` 命令中或 Dockerfile 中设置 `GITHUB_TOKEN` 环境变量。

1. 确保你已安装 Docker 并且 Docker 引擎正在运行。
2. 运行 `docker build ./ --tag komac_dev:latest`。
3. 等待构建完成。
4. 使用 `docker run -it komac_dev bash` 启动容器。
5. 测试任何命令。使用 `exit` 命令退出容器。