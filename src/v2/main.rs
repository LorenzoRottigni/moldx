#[tokio::main]
async fn main() -> Result<()> {
    // Init CLI parsing the incoming command (cli.command)
    let command = cli::parse_cli_command().ok_or_else(|| anyhow!("No command provided"))?;
    // create a new config considering strategies_dir override
    let config = MoldXConfig::new(command.strategies_dir.clone());
    // create a new client with the config
    let client = MoldXClient::new(config);

    client.exec(command).await?;


    // MoldXCLI:
    // - holds the shell command
    // - provides available commands (src/cli/commands)
    // - executes the command with the client

    // MoldXClient:
    // - holds the config
    // - resolves strategy commands and templates
    // - delegates command execution to the executor?
    // - TL;DR: provides the API for interacting with the .moldx dir and its contents

    // MoldXConfig:
    // - holds the configuration that moldx uses to run.
    // - or should be MoldXState including both user configs and runtime state like the received command?
}
