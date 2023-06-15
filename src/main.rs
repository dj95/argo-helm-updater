use anyhow::bail;
use clap::Parser;
use inquire::Confirm;
use kube::{Client, ResourceExt};
use kubernetes::{init_client, patch_application, Application, SourceSpec};
use log::{error, info};

use crate::{helm::HelmChart, kubernetes::list_applications};

mod helm;
mod kubernetes;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Namespace that holds all applications to check")]
    namespace: Option<String>,

    #[arg(short, long, help = "Context of the cluster to connect to")]
    context: Option<String>,

    #[arg(
        long,
        default_value_t = false,
        help = "Prompt to update the application directly in the cluster"
    )]
    update: bool,
}

pub async fn verify_helm_source(
    client: &Client,
    argo_application: &Application,
    source_spec: &SourceSpec,
    should_ask_for_update: bool,
) -> anyhow::Result<()> {
    let helm = HelmChart::try_from(source_spec.clone());

    if helm.is_err() {
        return Ok(());
    }

    let helm = helm.unwrap();
    let newest_version = helm.get_newer_version().await?;

    if let Some(newest_version) = newest_version {
        info!(
            "{} has new version {} instead of {}",
            argo_application.name_any(),
            newest_version,
            helm.revision
        );

        if should_ask_for_update {
            ask_for_update(client, argo_application, &helm, &newest_version).await?;
        }
    }

    Ok(())
}

async fn ask_for_update(
    client: &Client,
    argo_application: &Application,
    helm: &HelmChart,
    newest_version: &str,
) -> anyhow::Result<()> {
    let ans = Confirm::new(&format!(
        "Do you want to update {} from {} to {}?",
        argo_application.name_any(),
        helm.revision,
        newest_version,
    ))
    .with_default(false)
    .with_help_message("Don't forget to also update the argo files in your git repo!")
    .prompt();

    match ans {
        Ok(true) => {
            patch_application(client, argo_application, helm, newest_version).await?;

            info!("successfully update the application spec");

            Ok(())
        }
        Ok(false) => {
            info!("not updating the chart");

            Ok(())
        }
        Err(_) => bail!("cannot get user confirmation to update"),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let client = init_client(args.context, args.namespace).await?;

    let apps = list_applications(&client).await?;
    for a in apps {
        if !a.contains_helm() {
            continue;
        }

        if a.helm_in_source() {
            let result =
                verify_helm_source(&client, &a, &(a.clone()).spec.source.unwrap(), args.update)
                    .await;

            if result.is_err() {
                error!("cannot fetch update: {:?}", result.err().unwrap());
            }
        }

        if a.helm_in_sources() {
            for source in (a.clone()).spec.sources.unwrap() {
                let result = verify_helm_source(&client, &a, &source, args.update).await;

                if result.is_err() {
                    error!("cannot fetch update: {:?}", result.err().unwrap());
                }
            }
        }
    }

    Ok(())
}
