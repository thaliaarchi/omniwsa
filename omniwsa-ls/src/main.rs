use std::{
    error::Error,
    fs,
    io::{self, ErrorKind},
    process::exit,
};

use bstr::ByteSlice;
use lsp_server::{Connection, Message, Request, Response};
use lsp_types::{
    request::{Request as _, SemanticTokensFullRequest},
    InitializeParams, SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams,
    SemanticTokensResult, SemanticTokensServerCapabilities, ServerCapabilities,
};
use omniwsa::{
    dialects::{Dialect, Palaiologos},
    tokens::{
        comment::BlockCommentError,
        string::{CharError, StringError},
        GroupError, Token,
    },
};
use serde_json::{from_value as from_json, to_value as to_json};

// TODO:
// - Implement text document API, instead of reading from disk.
// - Record spans in tokens.

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
                work_done_progress_options: Default::default(),
                legend: SemanticTokensLegend {
                    token_types: vec![
                        SemanticTokenType::VARIABLE,
                        SemanticTokenType::FUNCTION,
                        SemanticTokenType::KEYWORD,
                        SemanticTokenType::COMMENT,
                        SemanticTokenType::STRING,
                        SemanticTokenType::NUMBER,
                        SemanticTokenType::OPERATOR,
                    ],
                    token_modifiers: vec![
                        SemanticTokenModifier::DECLARATION,
                        SemanticTokenModifier::DEFINITION,
                    ],
                },
                range: Some(false),
                full: Some(SemanticTokensFullOptions::Bool(true)),
            },
        )),
        ..Default::default()
    }
}

// The order corresponds to the index in SemanticTokensLegend::token_types.
#[derive(Clone, Copy, Debug)]
enum TokenType {
    Variable,
    Function,
    Keyword,
    Comment,
    String,
    Number,
    Operator,
}

// The order corresponds to the index in SemanticTokensLegend::token_modifiers.
#[derive(Clone, Copy, Debug)]
enum TokenModifier {
    Declaration,
    Definition,
}

fn main_loop(
    connection: Connection,
    _initialize_params: InitializeParams,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                eprintln!("Receive {req:?}");
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                let Request { id, method, params } = req;
                match &*method {
                    SemanticTokensFullRequest::METHOD => {
                        // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_semanticTokens
                        // https://code.visualstudio.com/api/language-extensions/semantic-highlight-guide
                        // https://github.com/rust-lang/rust-analyzer/blob/master/crates/rust-analyzer/src/lsp/semantic_tokens.rs
                        // https://github.com/rust-lang/rust-analyzer/blob/c4e040ea8dc4651569514bd8a8d8b293b49390a6/crates/rust-analyzer/src/lsp/capabilities.rs#L123

                        let params: SemanticTokensParams = from_json(params)?;
                        // TODO: Implement text document API, instead of reading
                        // from disk.
                        let path = params.text_document.uri.as_str();
                        let path = path.strip_prefix("file://").unwrap_or(path);
                        let src = fs::read(path)?;
                        let tokens = Palaiologos::new().lex(&src);

                        let mut tokens_out = Vec::with_capacity(tokens.len());
                        let (mut curr_line, mut curr_col) = (0, 0);
                        let (mut prev_line, mut prev_col) = (0, 0);
                        for tok in &tokens {
                            eprintln!("{tok:?}");
                            let ungrouped = tok.ungroup();
                            let ty = match ungrouped {
                                Token::Mnemonic(_) => Some(TokenType::Keyword),
                                Token::Integer(_) => Some(TokenType::Number),
                                Token::String(_) | Token::Char(_) => Some(TokenType::String),
                                Token::Variable(_) => Some(TokenType::Variable),
                                Token::Label(_) => Some(TokenType::Function),
                                Token::LabelColon(_) => Some(TokenType::Operator),
                                Token::Space(_) | Token::LineTerm(_) | Token::Eof(_) => None,
                                Token::InstSep(_) | Token::ArgSep(_) => Some(TokenType::Operator),
                                Token::LineComment(_) | Token::BlockComment(_) => {
                                    Some(TokenType::Comment)
                                }
                                Token::Word(_) => Some(TokenType::Variable),
                                Token::Group(_) | Token::Spliced(_) => panic!("not ungrouped"),
                                Token::Error(_) => None,
                                Token::Placeholder => panic!("placeholder"),
                            };
                            let modifiers = match ungrouped {
                                Token::Label(_) => TokenModifier::Declaration as _,
                                Token::Variable(_) => TokenModifier::Definition as _,
                                _ => 0,
                            };
                            let (len, hlen, vlen) = token_len(tok);
                            let (mut next_line, mut next_col) = (curr_line, curr_col);
                            if vlen != 0 {
                                next_col = 0;
                                next_line += vlen;
                            }
                            next_col += hlen;
                            if let Some(ty) = ty {
                                let token_out = SemanticToken {
                                    delta_line: (curr_line - prev_line) as _,
                                    delta_start: if curr_line == prev_line {
                                        (curr_col - prev_col) as _
                                    } else {
                                        curr_col as _
                                    },
                                    length: len as _,
                                    token_type: ty as _,
                                    token_modifiers_bitset: modifiers,
                                };
                                eprintln!("=> {token_out:?}");
                                tokens_out.push(token_out);
                                prev_line = curr_line;
                                prev_col = curr_col;
                            }
                            curr_line = next_line;
                            curr_col = next_col;
                        }

                        let result = Some(SemanticTokensResult::Tokens(SemanticTokens {
                            result_id: None,
                            data: tokens_out,
                        }));
                        let resp = Response {
                            id,
                            result: Some(to_json(&result)?),
                            error: None,
                        };
                        eprintln!("Send {resp:?}");
                        eprintln!();
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
                eprintln!("Receive {resp:?}");
            }
            Message::Notification(notif) => {
                eprintln!("Receive {notif:?}");
            }
        }
    }
    Ok(())
}

/// Computes the length of the token. Returns the linear, horizontal, vertical
/// lengths in chars.
// TODO: Record spans in tokens instead of this hack.
fn token_len(tok: &Token<'_>) -> (usize, usize, usize) {
    let (text, len_before, len_after): (&[u8], usize, usize) = match tok {
        Token::Mnemonic(tok) => (&tok.mnemonic, 0, 0),
        Token::Integer(tok) => (&tok.literal, 0, 0),
        Token::String(tok) => (
            &tok.literal,
            tok.quotes.quote().len(),
            if tok.errors.contains(StringError::Unterminated) {
                0
            } else {
                tok.quotes.quote().len()
            },
        ),
        Token::Char(tok) => (
            &tok.literal,
            tok.quotes.quote().len(),
            if tok.errors.contains(CharError::Unterminated) {
                0
            } else {
                tok.quotes.quote().len()
            },
        ),
        Token::Variable(tok) => (&tok.ident, tok.style.sigil().len(), 0),
        Token::Label(tok) => (&tok.label, tok.style.sigil().len(), 0),
        Token::LabelColon(_) => (b":", 0, 0),
        Token::Space(tok) => (&tok.space, 0, 0),
        Token::LineTerm(tok) => (tok.style.as_str().as_bytes(), 0, 0),
        Token::Eof(_) => (b"", 0, 0),
        Token::InstSep(tok) => (tok.style.as_str().as_bytes(), 0, 0),
        Token::ArgSep(tok) => (tok.style.as_str().as_bytes(), 0, 0),
        Token::LineComment(tok) => (tok.text, tok.style.prefix().len(), 0),
        Token::BlockComment(tok) => (
            tok.text,
            tok.style.open().len(),
            if tok.errors.contains(BlockCommentError::Unterminated) {
                0
            } else {
                tok.style.close().len()
            },
        ),
        Token::Word(tok) => (&tok.word, 0, 0),
        Token::Group(tok) => {
            let (len, hlen, vlen) = token_len(&tok.inner);
            let mut quotes = 0;
            if vlen != 0 {
                quotes += tok.delim.open().len();
            }
            if !tok.errors.contains(GroupError::Unterminated) {
                quotes += tok.delim.close().len();
            }
            return (len + quotes, hlen + quotes, vlen);
        }
        Token::Spliced(tok) => {
            let (mut spliced_len, mut spliced_hlen, mut spliced_vlen) = (0, 0, 0);
            for tok in &tok.tokens {
                let (len, hlen, vlen) = token_len(tok);
                if vlen != 0 {
                    spliced_hlen = 0;
                    spliced_vlen += vlen;
                }
                spliced_len += len;
                spliced_hlen += hlen;
            }
            return (spliced_len, spliced_hlen, spliced_vlen);
        }
        Token::Error(tok) => (&tok.text, 0, 0),
        Token::Placeholder => panic!("placeholder"),
    };
    let (len, hlen, vlen) =
        text.chars()
            .fold((len_before, len_before, 0), |(len, hlen, vlen), ch| {
                if ch == '\n' {
                    (len + 1, 0, vlen + 1)
                } else {
                    (len + 1, hlen + 1, vlen)
                }
            });
    (len + len_after, hlen + len_after, vlen)
}
