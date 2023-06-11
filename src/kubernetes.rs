use anyhow::Ok;
use kube::{
    api::{ListParams, Patch, PatchParams},
    client::ConfigExt,
    config::KubeConfigOptions,
    Api, Client, Config, ResourceExt,
};
use kube_derive::CustomResource;
use log::debug;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::helm::HelmChart;

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
        config.default_namespace = namespace;
    }

    let https = config.openssl_https_connector()?;
    let service = tower::ServiceBuilder::new()
        .layer(config.base_uri_layer())
        .option_layer(config.auth_layer()?)
        .service(hyper::Client::builder().build(https));

    Ok(Client::new(service, config.default_namespace))
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
    pub fn helm_in_source(&self) -> bool {
        if let Some(source) = &self.spec.source {
            return source.is_helm();
        }

        false
    }

    pub fn helm_in_sources(&self) -> bool {
        if let Some(sources) = &self.spec.sources {
            for source in sources {
                if !source.is_helm() {
                    continue;
                }

                return true;
            }
        }

        false
    }

    pub fn contains_helm(&self) -> bool {
        if self.helm_in_source() {
            return true;
        }

        if self.helm_in_sources() {
            return true;
        }

        false
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct SourceSpec {
    pub chart: Option<String>,
    #[serde(rename = "repoURL")]
    pub repo_url: Option<String>,
    #[serde(rename = "targetRevision")]
    pub target_revision: Option<String>,
    pub helm: Option<Value>,
    #[serde(rename = "ref")]
    pub reference: Option<Value>,
    pub path: Option<String>,
    pub kustomize: Option<Value>,
    pub directory: Option<Value>,
    pub plugin: Option<Value>,
}

impl SourceSpec {
    pub fn is_helm(&self) -> bool {
        self.chart.is_some()
    }
}

pub async fn list_applications(client: &Client) -> anyhow::Result<Vec<Application>> {
    let apps_api: Api<Application> = Api::default_namespaced(client.clone());
    let apps = apps_api.list(&ListParams::default()).await?;

    let mut output = Vec::new();

    for a in apps {
        output.push(a);
    }

    Ok(output)
}

pub async fn patch_application(
    client: &Client,
    argo_application: &Application,
    helm: &HelmChart,
    new_revision: &str,
) -> Result<Application, kube::Error> {
    let apps_api: Api<Application> = Api::default_namespaced(client.clone());

    let patch = get_application_patch(argo_application, helm, new_revision);

    debug!("{}", patch);

    apps_api
        .patch(
            &argo_application.name_any(),
            &PatchParams::apply("argo-helm-updater"),
            &Patch::Merge(patch),
        )
        .await
}

fn get_application_patch(
    argo_application: &Application,
    helm: &HelmChart,
    new_revision: &str,
) -> Value {
    if argo_application.helm_in_sources() {
        let sources = argo_application.spec.sources.clone().unwrap();
        let mut patched_sources: Vec<SourceSpec> = Vec::new();

        for source in sources {
            let mut patched_source = source.clone();

            if source.chart.is_some() && helm.chart.cmp(&(source.clone()).chart.unwrap()).is_eq() {
                patched_source.target_revision = Some((*new_revision).to_string());
            }

            patched_sources.push(patched_source);
        }

        return json!({ "spec": { "sources": patched_sources } });
    }

    let mut patched_source = argo_application.spec.source.clone().unwrap();

    if helm
        .chart
        .cmp(&(patched_source.clone()).chart.unwrap())
        .is_eq()
    {
        patched_source.target_revision = Some(new_revision.to_string());
    }

    json!({ "spec": { "source": patched_source } })
}
