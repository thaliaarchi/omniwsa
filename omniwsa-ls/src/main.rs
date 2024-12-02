use std::{
    error::Error,
    io::{self, ErrorKind},
    process::exit,
};

use lsp_server::{Connection, Message, Request, Response};
use lsp_types::{
    request::{Request as _, SemanticTokensFullRequest},
    InitializeParams, SemanticToken, SemanticTokens, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams, SemanticTokensResult,
    SemanticTokensServerCapabilities, ServerCapabilities, WorkDoneProgressOptions,
};
use serde_json::{from_value as from_json, to_value as to_json};

fn main() {
    if let Err(err) = do_main() {
        eprintln!("Error: {err}");
        exit(1);
    }
}

fn do_main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = to_json(&server_capabilities())?;
    let initialize_params: InitializeParams =
        from_json(connection.initialize(server_capabilities)?)?;

    main_loop(connection, initialize_params)?;
    io_threads.join()?;
    Ok(())
}

fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: Some(true), // TODO
                },
                legend: SemanticTokensLegend {
                    token_types: vec![],     // TODO
                    token_modifiers: vec![], // TODO
                },
                range: Some(false),
                full: Some(SemanticTokensFullOptions::Bool(true)),
            },
        )),
        ..Default::default()
    }
}

fn main_loop(
    connection: Connection,
    _initialize_params: InitializeParams,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                eprintln!(
                    "Received request {} #{}: {:?}",
                    req.method, req.id, req.params,
                );
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                let Request { id, method, params } = req;
                match &*method {
                    SemanticTokensFullRequest::METHOD => {
                        let _params: SemanticTokensParams = from_json(params)?;
                        let tokens = vec![SemanticToken {
                            delta_line: 1,
                            delta_start: 2,
                            length: 3,
                            token_type: 0,
                            token_modifiers_bitset: 0,
                        }];
                        let result = Some(SemanticTokensResult::Tokens(SemanticTokens {
                            result_id: None,
                            data: tokens,
                        }));
                        let resp = Response {
                            id,
                            result: Some(to_json(&result)?),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                    }
                    _ => {
                        return Err(Box::new(io::Error::new(
                            ErrorKind::Unsupported,
                            format!("unknown method {method}"),
                        )))
                    }
                }
            }
            Message::Response(resp) => {
                eprintln!("Received response: {resp:?}");
            }
            Message::Notification(notif) => {
                eprintln!("Received notification: {notif:?}");
            }
        }
    }
    Ok(())
}
