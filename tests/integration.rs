use std::str::FromStr;
use anyhow::{anyhow, bail, Result};

use drift::math::constants::{BASE_PRECISION_I64, LAMPORTS_PER_SOL_I64, PRICE_PRECISION_U64};
use drift_sdk::{
    get_market_accounts,
    types::{Context, MarketId, NewOrder, ClientOpts},
    DriftClient, RpcAccountProvider, Wallet,WsAccountProvider,
};
use solana_sdk::{signature::Keypair, signer::Signer};

/// keypair for integration tests
fn test_keypair() -> Keypair {
    // let private_key = std::env::var("TEST_PRIVATE_KEY").expect("TEST_PRIVATE_KEY set");
    // Keypair::from_base58_string(private_key.as_str())
    Keypair::from_base58_string("JmtV5CeqAfGLE4meNQTaBYRomXoXit58b4zTgfPJFufAkhxKX859CD4ufAV8yUnfZaEJs3KwYm4Nnw3uxLsypBd")
}

#[tokio::test]
async fn get_oracle_prices() {
    let client = DriftClient::new(
        Context::DevNet,
        RpcAccountProvider::new("https://api.devnet.solana.com"),
        Keypair::new().into(),
    )
    .await
    .expect("connects");
    let price = client.oracle_price(MarketId::perp(0)).await.expect("ok");
    assert!(price > 0);
    dbg!(price);
    let price = client.oracle_price(MarketId::spot(1)).await.expect("ok");
    assert!(price > 0);
    dbg!(price);
}

#[tokio::test]
async fn get_market_accounts_works() {
    let client = DriftClient::new(
        Context::DevNet,
        RpcAccountProvider::new("https://api.devnet.solana.com"),
        Keypair::new().into(),
    )
    .await
    .expect("connects");

    let (spot, perp) = get_market_accounts(client.inner()).await.unwrap();
    assert!(spot.len() > 1);
    assert!(perp.len() > 1);
}

#[tokio::test]
async fn place_and_cancel_orders() -> Result<()>{
    let wallet: Wallet = test_keypair().into();
    // let mut client = DriftClient::new(
    //     Context::MainNet,
    //     RpcAccountProvider::new(PROVIDER),
    //     wallet.clone(),
    // )
    // .await
    // .expect("connects");
        let mut client = DriftClient::new_with_opts(
        Context::MainNet,
        RpcAccountProvider::new(PROVIDER),
        wallet.clone(),
        ClientOpts::new(1, Some(vec![0,1])),
    )
    .await
    .expect("connects");

    let sol_perp = client.market_lookup("sol-perp").expect("exists");
    let sol_spot = client.market_lookup("sol").expect("exists");

    client.add_user(1).await.map_err(|e| anyhow!("e: {:?}", e))?;
    client.subscribe().await?;
    // println!("active_sub_account_id: {:?}", client.active_sub_account_id);
    // println!("get_user: {:?}", client.get_user(client.active_sub_account_id));

    let tx = client
        .init_tx(&wallet.sub_account(1), false)
        .unwrap()
        .cancel_all_orders()
        .place_orders(vec![
            NewOrder::limit(sol_perp)
                .amount(1 * BASE_PRECISION_I64)
                .price(40 * PRICE_PRECISION_U64)
                .post_only(drift_sdk::types::PostOnlyParam::TryPostOnly)
                .build(),
        ])
        .cancel_all_orders()
        .build();

    dbg!(tx.clone());

    let result = client.sign_and_send(tx).await;
    dbg!(&result);
    println!("xxxxx {:?}", result);
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn place_and_take() {
    let wallet: Wallet = test_keypair().into();
    let client = DriftClient::new(
        Context::DevNet,
        RpcAccountProvider::new("https://api.devnet.solana.com"),
        wallet.clone(),
    )
    .await
    .expect("connects");

    let sol_perp = client.market_lookup("sol-perp").expect("exists");

    let order = NewOrder::limit(sol_perp)
        .amount(1 * BASE_PRECISION_I64)
        .price(40 * PRICE_PRECISION_U64)
        .build();
    let tx = client
        .init_tx(&wallet.default_sub_account(), false)
        .unwrap()
        .place_and_take(order, None, None, None)
        .build();

    let result = client.sign_and_send(tx).await;
    dbg!(&result);
    // TODO: add a place and make to match against
    assert!(result.is_err());
}

const PROVIDER: &str = "http://solana-mainnet.core.chainstack.com/a2501887cb5c0b021117e5fccd1830b7";

pub use solana_sdk::{pubkey::Pubkey};
#[tokio::test]
async fn test_drift_client_readonly()  -> Result<()>{
    let owner_pubkey = Pubkey::from_str("C2qm4DwT289MDYW4nyy25SvGmaTrJqdgrWPKqjSktpmh")?;
    // let delegate_pubkey = Pubkey::from_str("Av6Mgrs9WPz67i8EEYf3dBKktmh6wAmy9zFYvoHEFAVA")?;

    let read_only_wallet: Wallet = Wallet::read_only(owner_pubkey);
    let sub_account: Pubkey = read_only_wallet.sub_account(1);

    println!("sub_account: {}", sub_account);

    let client = DriftClient::new(
        Context::MainNet,
        RpcAccountProvider::new(PROVIDER),
        read_only_wallet.clone(),
    )
    .await
    .expect("connects");

    let user_stats = client.get_user_stats(&owner_pubkey).await;
    println!("user_stats: {:?}\n\n", user_stats);

    let user_account = client.get_user_account(&sub_account).await?;
    println!("user_account: {:?}", user_account);

    println!("{:?}", user_account.perp_positions[0]);
    println!("{:?}", user_account.spot_positions[0]);


    println!("oracle_price: {}", client.oracle_price(MarketId::perp(1)).await?);
    Ok(())
}


#[tokio::test]
async fn test_drift_client_guest()  -> Result<()>{
    // // let delegate_private_key = "4gtLxmuFDC4WSuAU4hWaFfRjr4WJM11NBY1oJLXQFWvYYkfnvN9DsRuTfBwUgA6YXveZs4R5WnjCQ3ZquMA9AKaU".to_string();
    // // let delegate_key_pair = Keypair::from_base58_string(delegate_private_key.as_str());
    // let owner_pubkey = Pubkey::from_str("C2qm4DwT289MDYW4nyy25SvGmaTrJqdgrWPKqjSktpmh")?;

    // let read_only_wallet: Wallet = Wallet::read_only(owner_pubkey);
    // println!("is_delegated: {}", read_only_wallet.is_delegated());

    // println!("authority: {:?}", wallet.authority());
    // println!("{:?}", wallet);

    let client = DriftClient::new(
        Context::MainNet,
        RpcAccountProvider::new("http://solana-mainnet.core.chainstack.com/a2501887cb5c0b021117e5fccd1830b7"),
        Wallet::new(Keypair::from_bytes(&[0u8; 64]).unwrap()),
    )
    .await
    .expect("connects");

    
    for spot_market in  client.program_data().spot_market_configs()  {
        println!("spot_market: {:?}", spot_market);
        println!("{}", String::from_utf8(spot_market.name.to_vec())?);
    }
    println!("\n\n\n\n");
    for perp_market in  client.program_data().perp_market_configs()  {
        println!("perp_market: {:?}", perp_market);
        println!("{}", String::from_utf8(perp_market.name.to_vec())?);
    }

    Ok(())
}


#[tokio::test]
async fn test_drift_client_executer() -> Result<()> {
    let owner_pubkey = Pubkey::from_str("C2qm4DwT289MDYW4nyy25SvGmaTrJqdgrWPKqjSktpmh")?;
    let delegate_key_pair = Keypair::from_base58_string("4gtLxmuFDC4WSuAU4hWaFfRjr4WJM11NBY1oJLXQFWvYYkfnvN9DsRuTfBwUgA6YXveZs4R5WnjCQ3ZquMA9AKaU");
    let sub_account_id = 1;

    // let mut wallet = Wallet::new(delegate_key_pair.insecure_clone());
    // wallet.to_delegated(owner_pubkey);
    // let sub_account: Pubkey = wallet.sub_account(sub_account_id);

    // let client = DriftClient::new(
    //     Context::MainNet,
    //     RpcAccountProvider::new(PROVIDER),
    //     wallet.clone(),
    // )
    // .await
    // .expect("connects");

    // let sub_user_account = client.get_user_account(&sub_account).await?;

    // println!("sub_user_account: {:?}", sub_user_account);
    // assert!(sub_user_account.sub_account_id == sub_account_id);
    // assert!(sub_user_account.delegate == delegate_key_pair.pubkey());
    // println!("Ok");
    
    let mut wallet = Wallet::new(delegate_key_pair.insecure_clone());
    let client = DriftClient::new(
        Context::MainNet,
        RpcAccountProvider::new(PROVIDER),
        wallet.clone(),
    )
    .await
    .expect("connects");
    let tx = client
        .init_tx(&wallet.default_sub_account(), true)?;
    // println!("tx: {}", tx);

    Ok(())
}
// clear && RUSTUP_TOOLCHAIN=1.76.0 RUST_BACKTRACE=1 cargo test test_drift_client_readonly -- --nocapture


use drift_sdk::dlob_client::DLOBClient;

#[tokio::test]
async fn test_dlob_client()  -> Result<()>{
    let url = "https://dlob.drift.trade";
    let client = DLOBClient::new(url);
    let stream = client.subscribe_l2_book(MarketId::perp(0), None);
    let mut rx: tokio::sync::mpsc::Receiver<std::result::Result<drift_sdk::dlob_client::L2Orderbook, drift_sdk::types::SdkError>> = stream.into_rx();


    // 循环从 `rx` 中读取数据
    while let Some(result) = rx.recv().await {
        match result {
            Ok(orderbook) => {
                // 在这里处理接收到的订单簿数据
                println!("Received L2Orderbook: {:?}", orderbook);
            }
            Err(e) => {
                // 在这里处理接收到的错误
                println!("Error receiving L2Orderbook: {:?}", e);
            }
        }
    }
    Ok(())
}


use drift_sdk::event_subscriber::{EventSubscriber, DriftEventStream};
use drift_sdk::async_utils::retry_policy;
use futures_util::StreamExt; // 引入StreamExt扩展方法
use futures::stream::Stream; // 引入Stream trait

#[tokio::test]
async fn test_event_subscriber()  -> Result<()>{
    let owner_pubkey = Pubkey::from_str("C2qm4DwT289MDYW4nyy25SvGmaTrJqdgrWPKqjSktpmh")?;
    let delegate_key_pair = Keypair::from_base58_string("4gtLxmuFDC4WSuAU4hWaFfRjr4WJM11NBY1oJLXQFWvYYkfnvN9DsRuTfBwUgA6YXveZs4R5WnjCQ3ZquMA9AKaU");
    let sub_account_id = 1;
        let mut wallet = Wallet::new(delegate_key_pair.insecure_clone());
    wallet.to_delegated(owner_pubkey);
    let sub_account: Pubkey = wallet.sub_account(sub_account_id);

    let mut event_stream: DriftEventStream = EventSubscriber::subscribe(
        "wss://solana-mainnet.core.chainstack.com/a2501887cb5c0b021117e5fccd1830b7",
        sub_account,
        retry_policy::never(),
    )
    .await?;


    while let Some(event) = event_stream.next().await {
        println!("event: {:?}", event);
        dbg!(event);
    }
    Ok(())
}

use tokio::{
    sync::{
        watch::{self, Receiver},
    },
};
use solana_sdk::{account::Account,clock::Slot};

// #[tokio::test]
// async fn test_account_subscriber()  -> Result<()>{
//     let owner_pubkey = Pubkey::from_str("C2qm4DwT289MDYW4nyy25SvGmaTrJqdgrWPKqjSktpmh")?;
//     let delegate_key_pair = Keypair::from_base58_string("4gtLxmuFDC4WSuAU4hWaFfRjr4WJM11NBY1oJLXQFWvYYkfnvN9DsRuTfBwUgA6YXveZs4R5WnjCQ3ZquMA9AKaU");
//     let sub_account_id = 1;
//         let mut wallet = Wallet::new(delegate_key_pair.insecure_clone());
//     wallet.to_delegated(owner_pubkey);
//     let sub_account: Pubkey = wallet.sub_account(sub_account_id);

//     let mut account_provider = WsAccountProvider::new(
//         "wss://solana-mainnet.core.chainstack.com/a2501887cb5c0b021117e5fccd1830b7"
//     ).await?;
//     let (tx, mut rx) = watch::channel((Account, 0));

//     // while let Some(event) = event_stream.next().await {
//     //     println!("event: {:?}", event);
//     //     dbg!(event);
//     // }
//     Ok(())
// }



// clear && RUSTUP_TOOLCHAIN=1.76.0 RUST_BACKTRACE=1 cargo test test_drift_client_readonly -- --nocapture
