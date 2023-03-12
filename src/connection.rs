use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use tf_provider::{attribute_path::AttributePath, Attribute, Diagnostics};

#[derive(Debug, PartialEq, Eq)]
pub struct ExecutionResult {
    pub status: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[async_trait]
pub trait Connection: Send + Sync + 'static {
    const NAME: &'static str;

    /// execute a command over the connection
    async fn execute(
        &self,
        cmd: Vec<Vec<u8>>,
        env: HashMap<Vec<u8>, Vec<u8>>,
    ) -> Result<ExecutionResult>;

    /// Validate the state is valid
    async fn validate(&self, diags: &mut Diagnostics, attr_path: AttributePath) -> Option<()>;

    /// Get the schema for the connection block
    fn schema() -> HashMap<String, Attribute>;
}