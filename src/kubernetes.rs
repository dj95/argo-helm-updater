use anyhow::Ok;
use kube::{api::ListParams, client::ConfigExt, config::KubeConfigOptions, Api, Client, Config};
use kube_derive::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub async fn init_client(
    context: Option<String>,
    namespace: Option<String>,
) -> anyhow::Result<Client> {
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

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    group = "argoproj.io",
    version = "v1alpha1",
    kind = "Application",
    namespaced
)]
pub struct ApplicationSpec {
    pub project: String,
    pub source: Option<SourceSpec>,
    pub sources: Option<Vec<SourceSpec>>,
}

impl Application {
    pub fn contains_helm(&self) -> bool {
        if let Some(source) = &self.spec.source {
            return source.is_helm();
        }

        if let Some(sources) = &self.spec.sources {
            for source in sources {
                if !source.is_helm() {
                    continue;
                }

                return true;
            }
        }

        return false;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct SourceSpec {
    chart: Option<String>,
    #[serde(alias = "repoURL")]
    repo_url: Option<String>,
    #[serde(alias = "targetRevision")]
    target_revision: Option<String>,
}

impl SourceSpec {
    pub fn is_helm(&self) -> bool {
        return self.chart.is_some();
    }
}

pub async fn list_applications(client: &Client) -> anyhow::Result<Vec<Application>> {
    let apps_api: Api<Application> = Api::default_namespaced(client.clone());
    let apps = apps_api.list(&ListParams::default()).await?;

    let mut output = Vec::new();

    for a in apps {
        output.push(a);
    }

    return Ok(output);
}
