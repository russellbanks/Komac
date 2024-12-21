use clap::Parser;
use color_eyre::Result;

use crate::credential::handle_token;
use crate::github::github_client::GitHub;
use crate::manifests::print_changes;
use crate::types::package_identifier::PackageIdentifier;
use crate::types::package_version::PackageVersion;

/// 输出给定包和版本的清单
#[derive(Parser)]
pub struct ShowVersion {
    /// 包的唯一标识符
    #[arg()]
    package_identifier: PackageIdentifier,

    /// 包的版本
    #[arg(short = 'v', long = "version")]
    package_version: Option<PackageVersion>,

    /// 显示安装程序清单的开关
    #[arg(short, long)]
    installer_manifest: bool,

    /// 显示默认语言环境清单的开关
    #[arg(short, long = "defaultlocale-manifest")]
    default_locale_manifest: bool,

    /// 显示所有语言环境清单的开关
    #[arg(short, long)]
    locale_manifests: bool,

    /// 显示版本清单的开关
    #[arg(long)]
    version_manifest: bool,

    /// 具有 `public_repo` 范围的 GitHub 个人访问令牌
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl ShowVersion {
    pub async fn run(self) -> Result<()> {
        let token = handle_token(self.token.as_deref()).await?;
        let github = GitHub::new(&token)?;

        // 获取给定包的所有版本列表
        let mut versions = github.get_versions(&self.package_identifier).await?;

        // 获取最新或指定版本的清单
        let manifests = github
            .get_manifests(
                &self.package_identifier,
                &self
                    .package_version
                    .unwrap_or_else(|| versions.pop_last().unwrap_or_else(|| unreachable!())),
            )
            .await?;

        let all = matches!(
            (
                self.installer_manifest,
                self.default_locale_manifest,
                self.locale_manifests,
                self.version_manifest
            ),
            (false, false, false, false)
        );

        let mut contents = Vec::new();
        if all || self.installer_manifest {
            contents.push(serde_yaml::to_string(&manifests.installer)?);
        }
        if all || self.default_locale_manifest {
            contents.push(serde_yaml::to_string(&manifests.default_locale)?);
        }
        if all || self.locale_manifests {
            contents.extend(
                manifests
                    .locales
                    .into_iter()
                    .flat_map(|locale_manifest| serde_yaml::to_string(&locale_manifest)),
            );
        }
        if all || self.version_manifest {
            contents.push(serde_yaml::to_string(&manifests.version)?);
        }

        print_changes(contents.iter().map(String::as_str));

        Ok(())
    }
}
