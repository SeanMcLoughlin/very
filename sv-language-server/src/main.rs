use anyhow::Result;
use tokio::io::{stdin, stdout};
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() -> Result<()> {
    let stdin = stdin();
    let stdout = stdout();

    let (service, socket) = LspService::new(sv_language_server::create_backend);
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
