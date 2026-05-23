use async_trait::async_trait;
use oxpm_dep::{DepNode, DepType};
use smol_str::SmolStr;

#[async_trait]
pub trait Resolver: Send + Sync + 'static {
    type Source;

    async fn resolve(
        &self,
        name: &SmolStr,
        range: &str,
        dep_type: DepType,
    ) -> std::result::Result<DepNode, Box<dyn std::error::Error + Send + Sync>>;
}