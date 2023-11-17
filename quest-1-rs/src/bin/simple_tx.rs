use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::native_token::{lamports_to_sol, LAMPORTS_PER_SOL};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::{SeedDerivable, Signer};
use solana_sdk::system_instruction::create_account_with_seed;
use solana_sdk::system_program;
use solana_sdk::transaction::Transaction;
use tracing::{debug, error, info};

fn main() {
    tracing_subscriber::fmt::init();
    let rpc_url = std::env::var("RPC_URL").expect("Missing RPC_URL env var");

    // create json rpc client
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    // create keypair from seed
    let payer_keypair = Keypair::from_seed_phrase_and_passphrase("test_wallet", "")
        .expect("Error generate keypair");
    let payer = payer_keypair.pubkey();
    info!("Using payer address: {:#?}", payer);

    // request airdrop
    match client.request_airdrop(&payer, 1000 * LAMPORTS_PER_SOL) {
        Ok(sig) => loop {
            if let Ok(confirmed) = client.confirm_transaction(&sig) {
                if confirmed {
                    // debug!("Transaction: {} Status: {}", sig, confirmed);
                    break;
                }
            }
        },
        Err(_) => {
            error!("Error requesting airdrop");
        }
    }

    let current_balance = client
        .get_balance(&payer)
        .expect("Error fetching payer balance");
    info!(
        "Payer balance in SOL = {}, in lamports = {}",
        lamports_to_sol(current_balance),
        current_balance
    );

    let system_program_id = system_program::id();
    let seed = "test_program_001";

    let derived_pubkey = Pubkey::create_with_seed(&payer, seed, &system_program_id)
        .expect("Error creating derived pubkey");
    info!("Program id {}", derived_pubkey);

    if let Ok(created_program) = client.get_account(&derived_pubkey) {
        info!("Program is already created {:#?}", created_program);
        return;
    }

    // get rent exempt
    let data_length = 1500;
    let rent_exempt = client
        .get_minimum_balance_for_rent_exemption(data_length)
        .expect("Error get rent exempt");
    let space = 0;

    // create account
    let instr = create_account_with_seed(
        &payer,
        &derived_pubkey,
        &payer,
        seed,
        rent_exempt,
        space,
        &system_program_id,
    );

    let (recent_blockhash, _) = client
        .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())
        .expect("Error get recent blockhash");

    let tx = Transaction::new_signed_with_payer(
        &[instr],
        Some(&payer),
        &[&payer_keypair],
        recent_blockhash,
    );

    debug!("Sending tx... {:#?}", tx);
    client
        .send_and_confirm_transaction(&tx)
        .expect("Error sending create account tx");
}
