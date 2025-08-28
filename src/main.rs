// #[derive(Serialize, Deserialize, PartialEq, Schema, Debug, Copy, Clone)]
// pub enum FilterMode {
//     // HACK: Plain enum seems to cause issues with postcard/postcard-rpc (send\_resp from client -> server hangs)
//     Lowpass(u8),
//     Bandpass(u8),
// }

use postcard_rpc::{
    define_dispatch, endpoints, header::{VarHeader, VarSeqKind}, host_client::test_channels as client, server::{
        impls::test_channels::{
            dispatch_impl::{new_server, Settings, WireSpawnImpl, WireTxImpl}, ChannelWireRx, ChannelWireSpawn, ChannelWireTx
        }, Dispatch, SpawnContext
    }, topics, Key1, Key2, Key4
};

use postcard_schema::Schema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Serialize, Deserialize, PartialEq, Schema, Debug, Copy, Clone)]
pub enum FilterMode {
    // HACK: Plain enum seems to cause issues with postcard/postcard-rpc (send\_resp from client -> server hangs)
    Lowpass,
    Bandpass,
}

#[tokio::main]
async fn main() {
    println!("{ENDPOINT_LIST:#?}");
    for i in ENDPOINT_LIST.endpoints {
        if i.0 == "filter/mode" {
            println!("key8: hex{0:02X?} dec{0:?}", i.1.to_bytes());
            println!("key4: hex{0:02X?} dec{0:?}", Key4::from_key8(i.1).to_bytes());
            println!("key2: hex{0:02X?} dec{0:?}", Key2::from_key8(i.1).to_bytes());
            println!("key1: hex{0:02X?} dec{0:?}", Key1::from_key8(i.1).to_bytes());
        }
    }
    let (client_tx, server_rx) = mpsc::channel(16);
    let (server_tx, client_rx) = mpsc::channel(16);

    let app = SingleDispatcher::new(TestContext {}, ChannelWireSpawn {});

    let cwrx = ChannelWireRx::new(server_rx);
    let cwtx = ChannelWireTx::new(server_tx);
    let kkind = app.min_key_len();
    let mut server = new_server(
        app,
        Settings {
            tx: cwtx,
            rx: cwrx,
            buf: 1024,
            kkind,
        },
    );
    tokio::task::spawn(async move {
        server.run().await;
    });
    let cli = client::new_from_channels(client_tx, client_rx, VarSeqKind::Seq4);
    for _ in 0..4 {
        cli.send_resp::<FilterModeEndpoint>(&FilterMode::Lowpass).await.unwrap();
        cli.send_resp::<FilterModeEndpoint>(&FilterMode::Bandpass).await.unwrap();
    }
    println!(":)");
}

endpoints! {
    list = ENDPOINT_LIST;
    | EndpointTy            | RequestTy             | ResponseTy            | Path              | Cfg                    |
    | ----------            | ---------             | ----------            | ----              | ---                    |
    | FilterModeEndpoint    | FilterMode            | ()                    | "filter/mode"     |
}

topics! {
    list = TOPICS_IN_LIST;
    direction = postcard_rpc::TopicDirection::ToServer;
    | TopicTy       | MessageTy             | Path      | Cfg                           |
    | ----------    | ---------             | ----      | ---                           |
}

topics! {
    list = TOPICS_OUT_LIST;
    direction = postcard_rpc::TopicDirection::ToClient;
    | TopicTy           | MessageTy     | Path              |
    | ----------        | ---------     | ----              |
}

pub struct TestContext {}

pub struct TestSpawnContext {}

impl SpawnContext for TestContext {
    type SpawnCtxt = TestSpawnContext;

    fn spawn_ctxt(&mut self) -> Self::SpawnCtxt {
        TestSpawnContext {}
    }
}

define_dispatch! {
    app: SingleDispatcher;
    spawn_fn: spawn_fn;
    tx_impl: WireTxImpl;
    spawn_impl: WireSpawnImpl;
    context: TestContext;

    endpoints: {
        list: crate::ENDPOINT_LIST;

        | EndpointTy            | kind          | handler                   |
        | ----------            | ----          | -------                   |
        | FilterModeEndpoint    | async         | filter_hdlr               |
    };
    topics_in: {
        list: crate::TOPICS_IN_LIST;

        | TopicTy           | kind      | handler               |
        | ----------        | ----      | -------               |
    };
    topics_out: {
        list: TOPICS_OUT_LIST;
    };
}

async fn filter_hdlr(_context: &mut TestContext, _header: VarHeader, _body: FilterMode) {
    println!("yey");
}
