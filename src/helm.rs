use std::{cmp::Ordering, collections::HashMap};

use anyhow::{bail, Ok};
use chrono::{DateTime, Utc};
use log::debug;
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
    pub async fn get_newer_version(&self) -> anyhow::Result<Option<String>> {
        let newest_version = get_helm_repo_index(&self.repo)
            .await?
            .get_newest_chart_version(&self.chart)?;

        match self.revision.cmp(&newest_version) {
            Ordering::Greater => Ok(Some(newest_version)),
            Ordering::Equal => Ok(None),
            Ordering::Less => Ok(Some(newest_version)),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct HelmRepoChartVersion {
    #[serde(alias = "apiVersion")]
    pub api_version: Option<String>,
    pub name: String,
    pub version: String,
    pub created: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct HelmRepoIndex {
    #[serde(alias = "apiVersion")]
    pub api_version: String,
    pub entries: HashMap<String, Vec<HelmRepoChartVersion>>,
}

impl HelmRepoIndex {
    pub fn get_newest_chart_version(&self, chart_name: &str) -> anyhow::Result<String> {
        let mut versions = self.entries.get(chart_name).unwrap().clone();

        versions.sort_by(|a, b| a.created.cmp(&b.created).reverse());

        return match versions.first() {
            Some(version) => Ok(version.version.to_string()),
            None => bail!("cannot get newest version"),
        };
    }
}

pub async fn get_helm_repo_index(repo_url: &str) -> anyhow::Result<HelmRepoIndex> {
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
