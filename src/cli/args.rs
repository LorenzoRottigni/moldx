use anyhow::Result;
use std::io::{self, Write};

use crate::client::MoldXClient;

pub struct ArgsResolver<'a> {
    client: &'a MoldXClient,
}

impl<'a> ArgsResolver<'a> {
    pub fn new(client: &'a MoldXClient) -> Self {
        Self { client }
    }

    pub async fn required(
        &self,
        value: Option<String>,
        prompt: &str,
    ) -> Result<String> {
        if let Some(value) = value {
            return Ok(value);
        }

        self.stdin(prompt)
    }

    fn stdin(&self, prompt: &str) -> Result<String> {
        print!("{prompt}: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_owned();

        if input.is_empty() {
            anyhow::bail!("A value is required");
        }

        Ok(input)
    }
}