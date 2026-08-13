#[tokio::main]
async fn main() -> Result<()> {
    // Init CLI parsing the incoming command (cli.command)
    let cli = MoldXCLI::new();
    // create a new config considering strategies_dir override
    let config = MoldXConfig::new(
        if let Some(dir) = cli.command.flags.strategies_dir {
            Some(dir)
        } else {
            None
        }
    );
    // create a new client with the config
    let client = MoldXClient::new(config);
    // execute the command with the client
    cli.run(client).await?;


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
