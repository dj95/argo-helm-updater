use clap::Parser;
use kube::ResourceExt;
use kubernetes::{init_client, Application, SourceSpec};

use crate::{helm::HelmChart, kubernetes::list_applications};

mod helm;
mod kubernetes;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    namespace: Option<String>,

    #[arg(short, long)]
    context: Option<String>,
}

pub async fn verify_helm_source(
    argo_application: &Application,
    source_spec: &SourceSpec,
) -> anyhow::Result<()> {
    let helm = HelmChart::try_from(source_spec.clone());

    if helm.is_err() {
        return Ok(());
    }

    let helm = helm.unwrap();
    let newest_version = helm.get_newer_version().await?;

    if let Some(newest_version) = newest_version {
        println!(
            "{} has new version {} instead of {}",
            argo_application.name_any(),
            newest_version,
            helm.revision
        );

        // TODO: patch kubernetes manifest for new version
    }

    return Ok(());
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let client = init_client(args.context, args.namespace).await?;

    let apps = list_applications(&client).await?;
    for a in apps {
        if !a.contains_helm() {
            continue;
        }

        if a.helm_in_source() {
            verify_helm_source(&a, &(a.clone()).spec.source.unwrap()).await?;
        }

        if a.helm_in_sources() {
            for source in (a.clone()).spec.sources.unwrap() {
                verify_helm_source(&a, &source).await?;
            }
        }
    }

    return Ok(());
}
