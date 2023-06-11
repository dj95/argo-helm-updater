use clap::Parser;
use kube::ResourceExt;
use kubernetes::init_client;

use crate::kubernetes::list_applications;

mod kubernetes;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    namespace: Option<String>,

    #[arg(short, long)]
    context: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let client = init_client(args.context, args.namespace).await?;

    println!(":: applications");
    let apps = list_applications(&client).await?;
    for a in apps {
        if !a.contains_helm() {
            continue;
        }

        println!("found app {}", a.name_any());
        println!("found app {:?}", a.spec.source);
        println!("found app {:?}", a.spec.sources);
        println!("");
    }

    return Ok(());
}
