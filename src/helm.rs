use std::{cmp::Ordering, collections::HashMap};

use anyhow::{bail, Ok};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::debug;
use mockall::{predicate::*, *};
use serde::Deserialize;
use serde_yaml::Error;

use crate::kubernetes::SourceSpec;

#[derive(Debug)]
pub struct HelmChart {
    pub chart: String,
    pub repo: String,
    pub revision: String,
}

impl TryFrom<SourceSpec> for HelmChart {
    type Error = anyhow::Error;

    fn try_from(value: SourceSpec) -> Result<Self, Self::Error> {
        if value.chart.is_none() {
            bail!("missing chart");
        }

        if value.repo_url.is_none() {
            bail!("missing repo_url");
        }

        if value.target_revision.is_none() {
            bail!("missing target_revision url");
        }

        Ok(Self {
            chart: value.chart.unwrap(),
            repo: value.repo_url.unwrap(),
            revision: value.target_revision.unwrap(),
        })
    }
}

impl HelmChart {
    pub async fn get_newer_version(
        &self,
        client: &dyn HelmRepoClient,
    ) -> anyhow::Result<Option<String>> {
        let newest_version = client
            .get_helm_repo_index(&self.repo)
            .await?
            .get_newest_chart_version(&self.chart)?;

        match self.revision.cmp(&newest_version) {
            Ordering::Greater => Ok(Some(newest_version)),
            Ordering::Equal => Ok(None),
            Ordering::Less => Ok(Some(newest_version)),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct HelmRepoChartVersion {
    #[serde(alias = "apiVersion")]
    pub api_version: Option<String>,
    pub name: String,
    pub version: String,
    pub created: DateTime<Utc>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct HelmRepoIndex {
    #[serde(alias = "apiVersion")]
    pub api_version: String,
    pub entries: HashMap<String, Vec<HelmRepoChartVersion>>,
}

impl HelmRepoIndex {
    pub fn get_newest_chart_version(&self, chart_name: &str) -> anyhow::Result<String> {
        let versions = self.entries.get(chart_name);

        if versions.is_none() {
            bail!("cannot find chart");
        }

        let mut semvers: Vec<_> = versions
            .unwrap()
            .iter()
            .filter_map(|v| {
                let mut ver = v.version.clone();
                if v.version.starts_with('v') {
                    ver = ver[1..].to_string();
                }
                versions::SemVer::new(&ver).map(|sv| (sv, v.version.clone()))
            })
            .filter(|(v, _)| v.pre_rel.is_none())
            .collect();

        semvers.sort();
        semvers.reverse();

        match semvers.first() {
            Some(version) => Ok(version.1.to_string()),
            None => bail!("cannot get newest version"),
        }
    }
}

#[automock]
#[async_trait]
pub trait HelmRepoClient {
    async fn get_helm_repo_index(&self, repo_url: &str) -> anyhow::Result<HelmRepoIndex>;
}

pub struct HelmRepoReqwestClient {}

#[async_trait]
impl HelmRepoClient for HelmRepoReqwestClient {
    async fn get_helm_repo_index(&self, repo_url: &str) -> anyhow::Result<HelmRepoIndex> {
        let res = reqwest::get(format!("{}/index.yaml", repo_url))
            .await?
            .text()
            .await?;

        let values: Result<HelmRepoIndex, Error> = serde_yaml::from_str(&res);

        match values {
            core::result::Result::Ok(v) => Ok(v),
            Err(e) => {
                debug!("{:?}", e);

                bail!("cannot fetch or deserialize helm repo index")
            }
        }
    }
}

#[cfg(test)]
mod test {

    use std::{collections::HashMap, str::FromStr};

    use chrono::DateTime;

    use crate::{
        helm::{HelmRepoClient, HelmRepoReqwestClient},
        kubernetes::SourceSpec,
    };

    use super::{HelmChart, HelmRepoChartVersion, HelmRepoIndex, MockHelmRepoClient};

    fn init_source_spec(
        chart: Option<String>,
        repo_url: Option<String>,
        target_revision: Option<String>,
    ) -> SourceSpec {
        SourceSpec {
            chart,
            repo_url,
            target_revision,
            helm: None,
            reference: None,
            path: None,
            kustomize: None,
            directory: None,
            plugin: None,
        }
    }

    #[test]
    fn helm_chart_source_spec_try_from_success() {
        let source_spec = init_source_spec(
            Some("chart".to_owned()),
            Some("repo_url".to_owned()),
            Some("target_revision".to_owned()),
        );

        let result = HelmChart::try_from(source_spec);

        assert!(result.is_ok());
    }

    #[test]
    fn helm_chart_source_spec_try_from_missing_chart() {
        let source_spec = init_source_spec(
            None,
            Some("repo_url".to_owned()),
            Some("target_revision".to_owned()),
        );

        let result = HelmChart::try_from(source_spec);

        assert!(result.is_err());
    }

    #[test]
    fn helm_chart_source_spec_try_from_missing_repo_url() {
        let source_spec = init_source_spec(
            Some("chart".to_owned()),
            None,
            Some("target_revision".to_owned()),
        );

        let result = HelmChart::try_from(source_spec);

        assert!(result.is_err());
    }

    #[test]
    fn helm_chart_source_spec_try_from_missing_target_revision() {
        let source_spec =
            init_source_spec(Some("chart".to_owned()), Some("repo_url".to_owned()), None);

        let result = HelmChart::try_from(source_spec);

        assert!(result.is_err());
    }

    fn create_stub_client() -> MockHelmRepoClient {
        let mut stub_client = MockHelmRepoClient::new();

        let versions = vec![
            HelmRepoChartVersion {
                api_version: Some("api_version".to_owned()),
                name: "name".to_owned(),
                version: "v0.1.0".to_owned(),
                created: DateTime::from_str("2022-11-10T11:40:08.566983693Z").expect("wrong param"),
            },
            HelmRepoChartVersion {
                api_version: Some("api_version".to_owned()),
                name: "name".to_owned(),
                version: "v0.2.0".to_owned(),
                created: DateTime::from_str("2022-11-11T11:40:08.566983693Z").expect("wrong param"),
            },
        ];

        let mut entries = HashMap::new();
        entries.insert("chart".to_owned(), versions);

        stub_client
            .expect_get_helm_repo_index()
            .returning(move |_| {
                Ok(HelmRepoIndex {
                    api_version: "v1".to_owned(),
                    entries: entries.clone(),
                })
            });

        stub_client
    }

    #[tokio::test]
    async fn helm_chart_get_newer_version_has_newer_version() {
        let client = create_stub_client();

        let helm_chart = HelmChart {
            chart: "chart".to_owned(),
            repo: "repo".to_owned(),
            revision: "v0.1.0".to_owned(),
        };

        let result = helm_chart.get_newer_version(&client).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.is_some());
        assert_eq!("v0.2.0", value.unwrap());
    }

    #[tokio::test]
    async fn helm_chart_get_newer_version_is_newest_version() {
        let client = create_stub_client();

        let helm_chart = HelmChart {
            chart: "chart".to_owned(),
            repo: "repo".to_owned(),
            revision: "v0.2.0".to_owned(),
        };

        let result = helm_chart.get_newer_version(&client).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn helm_repo_index_get_newest_chart_version_invalid_chart_name() {
        let version = Vec::new();
        let mut entries = HashMap::new();
        entries.insert("chart".to_owned(), version);

        let index = HelmRepoIndex {
            api_version: "v1".to_owned(),
            entries,
        };

        let result = index.get_newest_chart_version("invalid_chart");

        assert!(result.is_err());
    }

    #[test]
    fn helm_repo_index_get_newest_chart_version_valid_chart_name() {
        let versions = vec![
            HelmRepoChartVersion {
                api_version: Some("api_version".to_owned()),
                name: "name".to_owned(),
                version: "v0.1.0".to_owned(),
                created: DateTime::from_str("2022-11-10T11:40:08.566983693Z").expect("wrong param"),
            },
            HelmRepoChartVersion {
                api_version: Some("api_version".to_owned()),
                name: "name".to_owned(),
                version: "v0.2.0".to_owned(),
                created: DateTime::from_str("2022-11-11T11:40:08.566983693Z").expect("wrong param"),
            },
        ];

        let mut entries = HashMap::new();
        entries.insert("chart".to_owned(), versions);

        let index = HelmRepoIndex {
            api_version: "v1".to_owned(),
            entries,
        };

        let result = index.get_newest_chart_version("chart");

        assert!(result.is_ok());
        assert_eq!("v0.2.0", result.unwrap());
    }

    #[test]
    fn helm_repo_index_get_newest_chart_version_valid_chart_name_with_sort() {
        let verions = vec![
            HelmRepoChartVersion {
                api_version: Some("api_version".to_owned()),
                name: "name".to_owned(),
                version: "v0.2.0".to_owned(),
                created: DateTime::from_str("2022-11-11T11:40:08.566983693Z").expect("wrong param"),
            },
            HelmRepoChartVersion {
                api_version: Some("api_version".to_owned()),
                name: "name".to_owned(),
                version: "v0.1.0".to_owned(),
                created: DateTime::from_str("2022-11-10T11:40:08.566983693Z").expect("wrong param"),
            },
        ];

        let mut entries = HashMap::new();
        entries.insert("chart".to_owned(), verions);

        let index = HelmRepoIndex {
            api_version: "v1".to_owned(),
            entries,
        };

        let result = index.get_newest_chart_version("chart");

        assert!(result.is_ok());
        assert_eq!("v0.2.0", result.unwrap());
    }

    #[tokio::test]
    async fn helm_repo_client_get_helm_repo_index_error_on_404() {
        let mut server = mockito::Server::new_async().await;

        let mock = server.mock("GET", "/index.yaml").with_status(404).create();

        let helm_repo_client = HelmRepoReqwestClient {};

        let result = helm_repo_client.get_helm_repo_index(&server.url()).await;

        mock.assert();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn helm_repo_client_get_helm_repo_index_error_on_invalid_body() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/index.yaml")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("I'm an invalid body.")
            .create();

        let helm_repo_client = HelmRepoReqwestClient {};

        let result = helm_repo_client.get_helm_repo_index(&server.url()).await;

        mock.assert();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn helm_repo_client_get_helm_repo_index_success() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/index.yaml")
            .with_status(200)
            .with_header("content-type", "application/yaml")
            .with_body(
                "apiVersion: v1
entries:
  cert-manager:
  - apiVersion: v1
    created: \"2023-06-20T18:03:27.348311421Z\"
    name: name
    version: v0.2.0",
            )
            .create();

        let helm_repo_client = HelmRepoReqwestClient {};

        let result = helm_repo_client.get_helm_repo_index(&server.url()).await;

        mock.assert();
        assert!(result.is_ok());

        let value = result.unwrap();

        let versions = vec![HelmRepoChartVersion {
            api_version: Some("v1".to_owned()),
            name: "name".to_owned(),
            version: "v0.2.0".to_owned(),
            created: DateTime::from_str("2023-06-20T18:03:27.348311421Z").expect("wrong param"),
        }];

        let mut entries = HashMap::new();
        entries.insert("cert-manager".to_owned(), versions);

        let expected_value = HelmRepoIndex {
            api_version: "v1".to_owned(),
            entries,
        };
        assert_eq!(expected_value, value);
    }
}
