use std::{collections::HashMap, mem};

use color_eyre::eyre::Result;
use futures_util::{StreamExt, TryStreamExt, stream};
use winget_types::{installer::Architecture, url::DecodedUrl};

use crate::{download::DownloadedFile, file_analyser::FileAnalyser};

pub async fn process_files(
    files: &mut [DownloadedFile],
) -> Result<HashMap<DecodedUrl, FileAnalyser>> {
    stream::iter(files.iter_mut().map(
        |DownloadedFile {
             url,
             mmap,
             override_arch,
             sha_256,
             file_name,
             last_modified,
             ..
         }| async move {
            let mut file_analyser = FileAnalyser::new(mmap, file_name)?;
            let architecture = override_arch.or_else(|| Architecture::from_url(url.as_str()));
            for installer in &mut file_analyser.installers {
                if let Some(architecture) = architecture {
                    installer.architecture = architecture;
                }
                installer.url = url.clone();
                installer.sha_256 = sha_256.clone();
                installer.release_date = *last_modified;
            }
            file_analyser.file_name = mem::take(file_name);
            Ok((mem::take(url), file_analyser))
        },
    ))
    .buffer_unordered(num_cpus::get())
    .try_collect::<HashMap<_, _>>()
    .await
}
