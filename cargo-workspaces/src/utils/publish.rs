//! Helper functions useful when publishing (or preparing for publishing) crates.

use std::convert::TryFrom;

use crate::utils::{cargo_config_get, is_private, Error, Result};

use camino::Utf8PathBuf;
use cargo_metadata::{Metadata, Package};
use clap::Parser;
use indexmap::IndexSet as Set;
use tame_index::{
    external::{
        http::{HeaderMap, HeaderValue},
        reqwest::{blocking::Client, header::AUTHORIZATION, Certificate},
    },
    index::{ComboIndex, ComboIndexCache, RemoteGitIndex, RemoteSparseIndex},
    utils::flock::LockOptions,
    IndexLocation, IndexUrl, KrateName,
};

#[derive(Debug, Parser)]
#[clap(next_help_heading = "REGISTRY OPTIONS")]
pub struct RegistryOpt {
    /// The token to use for accessing the registry
    #[clap(long, forbid_empty_values(true))]
    pub token: Option<String>,

    /// The Cargo registry to use
    #[clap(long, forbid_empty_values(true))]
    pub registry: Option<String>,
}

pub fn filter_private(visited: Set<Utf8PathBuf>, pkgs: &[(Package, String)]) -> Set<Utf8PathBuf> {
    visited
        .into_iter()
        .filter(|x| {
            pkgs.iter()
                .find(|(p, _)| p.manifest_path == *x)
                .map(|(pkg, _)| !is_private(pkg))
                .unwrap_or(false)
        })
        .collect()
}

pub fn package_registry<'a>(
    metadata: &Metadata,
    registry: Option<&'a String>,
    pkg: &Package,
) -> Result<IndexUrl<'a>> {
    let url = if let Some(registry) =
        registry.or_else(|| pkg.publish.as_deref().and_then(|x| x.first()))
    {
        let registry_url = cargo_config_get(
            &metadata.workspace_root,
            &format!("registries.{}.index", registry),
        )?;
        IndexUrl::NonCratesIo(registry_url.into())
    } else {
        IndexUrl::crates_io(None, None, None)?
    };

    Ok(url)
}

pub fn create_http_client(workspace_root: &Utf8PathBuf, token: &Option<String>) -> Result<Client> {
    let client_builder = Client::builder().use_rustls_tls();
    let client_builder = if let Some(ref token) = token {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(token).unwrap());
        client_builder.default_headers(headers)
    } else {
        client_builder
    };
    let http_cainfo = cargo_config_get(workspace_root, "http.cainfo").ok();
    let client_builder = if let Some(http_cainfo) = http_cainfo {
        client_builder
            .tls_built_in_root_certs(false)
            .add_root_certificate(Certificate::from_pem(&std::fs::read(http_cainfo)?)?)
    } else {
        client_builder
    };
    Ok(client_builder.build()?)
}

pub fn is_published(
    client: &Client,
    index_url: IndexUrl,
    name: &str,
    version: &str,
) -> Result<bool> {
    let index_cache = ComboIndexCache::new(IndexLocation::new(index_url))?;
    let lock = LockOptions::cargo_package_lock(None)?.try_lock()?;

    let index: ComboIndex = match index_cache {
        ComboIndexCache::Git(git) => {
            let mut rgi = RemoteGitIndex::new(git, &lock)?;

            rgi.fetch(&lock)?;
            rgi.into()
        }
        ComboIndexCache::Sparse(sparse) => RemoteSparseIndex::new(sparse, client.clone()).into(),
        _ => return Err(Error::UnsupportedCratesIndexType),
    };

    let index_crate = index.krate(KrateName::try_from(name)?, false, &lock);
    match index_crate {
        Ok(Some(crate_data)) => Ok(crate_data.versions.iter().any(|v| v.version == version)),
        Ok(None) | Err(tame_index::Error::NoCrateVersions) => Ok(false),
        Err(e) => Err(e.into()),
    }
}
