use rand::Fill;
use std::{sync::atomic::AtomicU32, time::Duration};

use creditcoin_runtime::runtime_types::{
    frame_support::storage::bounded_vec::BoundedVec, pallet_creditcoin::types::Blockchain,
};
use goose::prelude::*;
use sp_keyring::AccountKeyring;
use subxt::{ClientBuilder, DefaultConfig, DefaultExtra, PairSigner};

use crate::creditcoin_runtime::DefaultAccountData;

#[subxt::subxt(runtime_metadata_path = "creditcoin-metadata.scale")]
pub mod creditcoin_runtime {}

static NONCE: AtomicU32 = AtomicU32::new(1);

type RuntimeApi = creditcoin_runtime::RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>;

async fn loadtest_register_address(user: &mut GooseUser) -> GooseTaskResult {
    let api = user.get_session_data_unchecked::<RuntimeApi>();
    let mut signer = PairSigner::new(AccountKeyring::Alice.pair());

    let mut addr = [0u8; 16];
    addr.try_fill(&mut rand::thread_rng()).unwrap();
    signer.set_nonce(NONCE.fetch_add(1, std::sync::atomic::Ordering::AcqRel));
    if let Err(e) = api
        .tx()
        .creditcoin()
        .register_address(Blockchain::Ethereum, BoundedVec(addr.to_vec()))
        .sign_and_submit(&signer)
        .await
    {
        eprintln!("Error: {}", e);
        drop(api);
        init_client(user).await?;
    }
    Ok(())
}

async fn init_client(user: &mut GooseUser) -> GooseTaskResult {
    let api = ClientBuilder::new().set_url("wss://qa.creditcoin.network:443").build().await.unwrap().to_runtime_api::<creditcoin_runtime::RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>>();
    user.set_session_data(api);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    let api = ClientBuilder::new().set_url("wss://qa.creditcoin.network:443").build().await.unwrap().to_runtime_api::<creditcoin_runtime::RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>>();
    let init_nonce = api
        .client
        .fetch_nonce::<DefaultAccountData>(&AccountKeyring::Alice.to_account_id())
        .await
        .unwrap();
    NONCE.store(init_nonce, std::sync::atomic::Ordering::Relaxed);
    GooseAttack::initialize()?
        .register_taskset(
            taskset!("LoadTestTasks")
                .set_wait_time(Duration::from_secs(0), Duration::from_secs(1))?
                .register_task(task!(init_client).set_on_start().set_name("Client init"))
                .register_task(task!(loadtest_register_address).set_name("RegisterAddress")),
        )
        .execute()
        .await?
        .print();
    Ok(())
}
