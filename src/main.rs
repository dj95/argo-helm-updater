use clap::Parser;
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::ListParams, client::ConfigExt, config::KubeConfigOptions, Api, Client, Config, ResourceExt,
};

async fn init_client(context: Option<String>, namespace: Option<String>) -> anyhow::Result<Client> {
    let mut config = Config::from_kubeconfig(&KubeConfigOptions {
        context,
        cluster: None,
        user: None,
    })
    .await?;

    if let Some(namespace) = namespace {
        config.default_namespace = String::from(namespace);
    }

    let https = config.openssl_https_connector()?;
    let service = tower::ServiceBuilder::new()
        .layer(config.base_uri_layer())
        .option_layer(config.auth_layer()?)
        .service(hyper::Client::builder().build(https));

    return Ok(Client::new(service, config.default_namespace));
}

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

    let pods: Api<Pod> = Api::default_namespaced(client);
    for p in pods.list(&ListParams::default()).await? {
        println!("found pod {}", p.name_any());
    }

    return Ok(());
}
