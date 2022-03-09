use std::{
    sync::{atomic::AtomicU32, Arc},
    time::Duration,
};

use creditcoin_runtime::runtime_types::{
    frame_support::storage::bounded_vec::BoundedVec, pallet_creditcoin::types::Blockchain,
};
use goose::{
    metrics::{GooseRawRequest, GooseRequestMetric},
    prelude::*,
};
use rand::distributions::{Alphanumeric, DistString};
use sp_keyring::AccountKeyring;
use subxt::{ClientBuilder, DefaultConfig, DefaultExtra, PairSigner};

use crate::creditcoin_runtime::DefaultAccountData;

#[subxt::subxt(runtime_metadata_path = "creditcoin-metadata.scale")]
pub mod creditcoin_runtime {}

static NONCE: AtomicU32 = AtomicU32::new(1);

type RuntimeApi = creditcoin_runtime::RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>;

async fn loadtest_register_address(api: &RuntimeApi, user: &mut GooseUser) -> GooseTaskResult {
    let mut signer = PairSigner::new(AccountKeyring::Alice.pair());
    signer.set_nonce(NONCE.fetch_add(1, std::sync::atomic::Ordering::AcqRel));

    let addr = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

    api.tx()
        .creditcoin()
        .register_address(Blockchain::Ethereum, BoundedVec(addr.into_bytes()))
        .sign_and_submit(&signer)
        .await
        .map_err(|e| {
            eprintln!("rpc error: {}", e);
            GooseTaskError::RequestFailed {
                raw_request: GooseRequestMetric {
                    elapsed: 0 as u64,
                    raw: GooseRawRequest {
                        method: GooseMethod::Put,
                        url: "wss://qa.creditcoin.network:443".into(),
                        headers: vec![],
                        body: String::new(),
                    },
                    name: "RegisterAddress".to_string(),
                    final_url: "".to_string(),
                    redirected: false,
                    response_time: 0,
                    status_code: 0,
                    success: false,
                    update: false,
                    user: user.weighted_users_index,
                    error: "".to_string(),
                    coordinated_omission_elapsed: 0,
                    user_cadence: 0,
                },
            }
        })?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    let api = Arc::new(
        ClientBuilder::new()
            .set_url("wss://qa.creditcoin.network:443")
            .build()
            .await
            .unwrap()
            .to_runtime_api::<RuntimeApi>(),
    );
    let init_nonce = api
        .client
        .fetch_nonce::<DefaultAccountData>(&AccountKeyring::Alice.to_account_id())
        .await
        .unwrap();

    NONCE.store(init_nonce, std::sync::atomic::Ordering::Relaxed);

    let closure: GooseTaskFunction = Arc::new(move |user| {
        let api = api.clone();
        Box::pin(async move { loadtest_register_address(&api, user).await })
    });

    let task = GooseTask::new(closure);
    let taskset = taskset!("LoadTestTasks")
        .set_wait_time(Duration::from_secs(1), Duration::from_secs(2))?
        .register_task(task);

    GooseAttack::initialize()?
        .register_taskset(taskset)
        .execute()
        .await?
        .print();
    Ok(())
}
