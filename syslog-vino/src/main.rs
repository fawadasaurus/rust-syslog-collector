// !!START_LINTS
// Vino lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![deny(
    clippy::expect_used,
    clippy::explicit_deref_methods,
    clippy::option_if_let_else,
    clippy::await_holding_lock,
    clippy::cloned_instead_of_copied,
    clippy::explicit_into_iter_loop,
    clippy::flat_map_option,
    clippy::fn_params_excessive_bools,
    clippy::implicit_clone,
    clippy::inefficient_to_string,
    clippy::large_types_passed_by_value,
    clippy::manual_ok_or,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::must_use_candidate,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::redundant_else,
    clippy::semicolon_if_nothing_returned,
    clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnested_or_patterns,
    clippy::future_not_send,
    clippy::useless_let_if_seq,
    clippy::str_to_string,
    clippy::inherent_to_string,
    clippy::let_and_return,
    clippy::string_to_string,
    clippy::try_err,
    clippy::unused_async,
    clippy::missing_enforced_import_renames,
    clippy::nonstandard_macro_braces,
    clippy::rc_mutex,
    clippy::unwrap_or_else_default,
    clippy::manual_split_once,
    clippy::derivable_impls,
    clippy::needless_option_as_deref,
    clippy::iter_not_returning_iterator,
    clippy::same_name_method,
    clippy::manual_assert,
    clippy::non_send_fields_in_send_ty,
    clippy::equatable_if_let,
    bad_style,
    clashing_extern_declarations,
    const_err,
    dead_code,
    deprecated,
    explicit_outlives_requirements,
    improper_ctypes,
    invalid_value,
    missing_copy_implementations,
    missing_debug_implementations,
    mutable_transmutes,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    trivial_bounds,
    trivial_casts,
    trivial_numeric_casts,
    type_alias_bounds,
    unconditional_recursion,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    while_true,
    missing_docs
)]
#![allow(unused_attributes)]
// !!END_LINTS
// Add exceptions here
#![allow(missing_docs, clippy::expect_used, unused)]

use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use syslog_rfc5424::SyslogMessage;
use syslog_rfc5424::parse_message;
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, UdpSocket},
};

#[derive(Parser, Debug, Clone)]
#[clap(name = "vino-syslog", about = "Vino syslog")]
pub(crate) struct CliArguments {
    #[clap()]
    pub(crate) manifest: String,
}

use tokio_stream::StreamExt;
use tracing::*;
use vino_host::Host;
use vino_transport::{MessageTransport, TransportMap};
use serde_json;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = CliArguments::parse();

    info!("manifest file:{}", cli.manifest);

    trace!("starting");
    let ip = "0.0.0.0";
    let tcp_port = "8468";
    let udp_port = "8514";
    let tcp_address = format!("{}:{}", ip, tcp_port);
    let udp_address = format!("{}:{}", ip, udp_port);
    let tcp_listener = TcpListener::bind(&tcp_address).await?;

    let builder = vino_host::HostBuilder::from_manifest_url(&cli.manifest, true, &[]).await?;
    let mut host = builder.build();
    host.start(None).await;
    let handler = MessageHandler::new(host);

    info!("TCPServer listening on  {}", tcp_address);
    let tcp_message_handler = handler.clone();
    let tcp_handle =
        tokio::spawn(async move { handle_tcp_client(tcp_listener, tcp_message_handler).await });

    let socket = UdpSocket::bind(&udp_address).await?;
    info!("UDP listening on {}", udp_address);
    let udp_handle = tokio::spawn(async move { handle_udp_client(socket, handler).await });

    tcp_handle.await?;

    Ok(())
}

#[derive(Clone, Debug)]
struct MessageHandler {
    host: Arc<Host>,
}

impl MessageHandler {
    fn new(host: Host) -> Self {
        Self {
            host: Arc::new(host),
        }
    }

    async fn handle_message(&self, row: &str) -> Result<()> {
        //let message = row.parse::<SyslogMessage>()?;
        let message = parse_message(row).unwrap();
        info!("{:?}", message);
        let message_json = serde_json::to_string(&message).unwrap();
        info!("{:?}", message_json);

        let payload = TransportMap::from([("input", message_json)]);
        let mut stream = self.host.request("echo", payload, None).await?;
        let response: Vec<_> = stream.collect_port("output").await;

        for msg in response {
            let output: String = msg.payload.deserialize()?;
            info!("{:?}", output);
        }

        Ok(())
    }
}

async fn handle_udp_client(socket: UdpSocket, handler: MessageHandler) -> Result<()> {
    let mut buf = [0; 4096];
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((amt, _src)) => {
                let buf = &mut buf[..amt];
                let row = std::str::from_utf8(buf).unwrap();
                handler.handle_message(row).await?;
            }
            Err(err) => {
                eprintln!("Err: {}", err);
            }
        }
    }
}

async fn handle_tcp_client(tcp_listener: TcpListener, handler: MessageHandler) -> Result<()> {
    while let Ok((mut stream, _)) = tcp_listener.accept().await {
        let mut data = String::new();
        //println!("New connection: {}", stream.peer_addr().unwrap());
        stream.read_to_string(&mut data).await?;
        handler.handle_message(&data).await?;
    }

    Ok(())
}
