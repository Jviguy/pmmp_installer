use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, Result};
use tokio::process::Command;

#[tokio::main(worker_threads = 2)]
async fn main() -> Result<()> {
    let matches = App::new("PocketMine-MP Installer")
        .version("0.1.0")
        .author("Jeremy I. jviguytwo2@gmail.com")
        .about("A CLI installer for the php minecraft bedrock server software PocketMine-MP")
        .args(&[
            Arg::with_name("directory")
                .short("d")
                .long("directory")
                .takes_value(true)
                .value_name("INSTALL_DIRECTORY")
                .default_value("./"),
            Arg::with_name("version")
                .short("v")
                .long("version")
                .takes_value(true)
                .value_name("VERSION")
                .default_value("Stable"),
            Arg::with_name("php-version")
                .short("p")
                .long("php-version")
                .takes_value(true)
                .value_name("PHP_VERSION")
                .default_value("7.4"),
        ])
        .get_matches();
    tokio::fs::create_dir_all(matches.value_of("directory").unwrap()).await?;
    let (phpm, pmmpm) = (matches.clone(), matches.clone());
    let downloads = vec![
        tokio::spawn(async move {
            download_php(
                phpm.value_of("php-version").unwrap(),
                phpm.value_of("directory").unwrap(),
            )
            .await
        }),
        tokio::spawn(async move {
            download_pmmp(
                pmmpm.value_of("version").unwrap(),
                pmmpm.value_of("directory").unwrap(),
            )
            .await
        }),
    ];
    for result in futures::future::join_all(downloads).await {
        result??;
    }
    match std::env::consts::OS {
        "windows" => {
            Command::new(format!(
                "{}/start.cmd",
                matches.value_of("directory").unwrap()
            ))
            .status()
            .await?;
        }
        _ => {
            Command::new(format!(
                "{}/start.sh",
                matches.value_of("directory").unwrap()
            ))
            .status()
            .await?;
        }
    };
    Ok(())
}

async fn download_pmmp(version: &str, path: &str) -> Result<()> {
    let pb = ProgressBar::new(100)
        .with_style(ProgressStyle::default_bar().template("{wide_bar} {pos}/{len} {msg}"));
    pb.inc(25);
    pb.set_message("Downloading the PocketMine-MP.phar...");
    let resp = if version == "Stable" {
        reqwest::get("https://jenkins.pmmp.io/job/PocketMine-MP/Stable/artifact/PocketMine-MP.phar")
            .await
            .unwrap()
    } else {
        reqwest::get(
            format!(
                "https://github.com/pmmp/PocketMine-MP/releases/download/{}/PocketMine-MP.phar",
                version
            )
            .as_str(),
        )
        .await
        .unwrap()
    };
    pb.inc(25);
    pb.set_message("Writing buffer into local file...");
    File::create(format!("{}/PocketMine-MP.phar", path))
        .await?
        .write_all(&*resp.bytes().await.unwrap())
        .await?;
    pb.inc(25);
    pb.set_message("Downloading needed start commands for this OS...");
    match std::env::consts::OS {
        "windows" => {
            let (resp, presp) = if version == "Stable" {
                (
                    reqwest::get(
                        "https://jenkins.pmmp.io/job/PocketMine-MP/Stable/artifact/start.cmd",
                    )
                    .await
                    .unwrap(),
                    reqwest::get(
                        "https://jenkins.pmmp.io/job/PocketMine-MP/Stable/artifact/start.ps1",
                    )
                    .await
                    .unwrap(),
                )
            } else {
                (
                    reqwest::get(
                        format!(
                            "https://github.com/pmmp/PocketMine-MP/releases/download/{}/start.cmd",
                            version
                        )
                        .as_str(),
                    )
                    .await
                    .unwrap(),
                    reqwest::get(
                        format!(
                            "https://github.com/pmmp/PocketMine-MP/releases/download/{}/start.ps1",
                            version
                        )
                        .as_str(),
                    )
                    .await
                    .unwrap(),
                )
            };
            File::create(format!("{}/start.cmd", path))
                .await?
                .write_all(&*resp.bytes().await.unwrap())
                .await?;
            File::create(format!("{}/start.ps1", path))
                .await?
                .write(&*presp.bytes().await.unwrap())
                .await?;
        }
        _ => {
            let resp = if version == "Stable" {
                reqwest::get("https://jenkins.pmmp.io/job/PocketMine-MP/Stable/artifact/start.sh")
                    .await
                    .unwrap()
            } else {
                reqwest::get(
                    format!(
                        "https://github.com/pmmp/PocketMine-MP/releases/download/{}/start.sh",
                        version
                    )
                    .as_str(),
                )
                .await
                .unwrap()
            };
            File::create(format!("{}/start.sh", path))
                .await?
                .write_all(&*resp.bytes().await.unwrap())
                .await?;
        }
    };
    pb.inc(25);
    pb.finish_and_clear();
    Ok(())
}

async fn download_php(version: &str, path: &str) -> Result<()> {
    let pb = ProgressBar::new(100)
        .with_style(ProgressStyle::default_bar().template("{wide_bar} {pos}/{len} {msg}"));
    pb.inc(25);
    pb.set_message("Downloading the php aggregate...");
    let (resp, filename, mut file) = match std::env::consts::OS {
        "windows" => (reqwest::get(format!(
            "https://jenkins.pmmp.io/job/PHP-{}-Aggregate/lastSuccessfulBuild/artifact/PHP-{}-{}-{}.{}",
            version, version, "Windows", "x64", "zip"
        ).as_str()).await.unwrap(),
                      format!("{}/PHP-{}-{}-{}.{}", path, version, "Windows", "x64", "zip"),
                      File::create(format!("{}/PHP-{}-{}-{}.{}", path, version, "Windows", "x64", "zip")).await?),
        "macos" => (reqwest::get(format!(
            "https://jenkins.pmmp.io/job/PHP-{}-Aggregate/lastSuccessfulBuild/artifact/PHP-{}-{}-{}.{}",
            version, version, "MacOS", "x86_64", "tar.gz"
        ).as_str()).await.unwrap(),
                    format!("{}/PHP-{}-{}-{}.{}", path, version, "MacOS", "x86_64", "tar.gz"),
                    File::create(format!("{}/PHP-{}-{}-{}.{}", path, version, "MacOS", "x86_64", "tar.gz")).await?),
        //linux because yeah lol...
        _ => (reqwest::get(format!(
            "https://jenkins.pmmp.io/job/PHP-{}-Aggregate/lastSuccessfulBuild/artifact/PHP-{}-{}-{}.{}",
            version, version, "Linux", "x86_64", "tar.gz"
        ).as_str()).await.unwrap(),
              format!("{}/PHP-{}-{}-{}.{}", path, version, "Linux", "x86_64", "tar.gz"),
              File::create(format!("{}/PHP-{}-{}-{}.{}", path, version, "Linux", "x86_64", "tar.gz")).await?)
    };
    pb.inc(25);
    pb.set_message("Writing buffer into local file...");
    file.write_all(&*resp.bytes().await.unwrap()).await?;
    pb.inc(25);
    pb.set_message("Extracting php binary and removing temporary archive..");
    archiver_rs::open(Path::new(filename.as_str()))
        .unwrap()
        .extract(path.as_ref())
        .unwrap();
    tokio::fs::remove_file(filename).await?;
    if std::env::consts::OS == "windows" {
        Command::new(format!("{}/vc_redist.x64.exe", path))
            .output()
            .await?;
    };
    pb.inc(25);
    pb.finish_and_clear();
    Ok(())
}
