use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::message::Message;
use solana_sdk::native_token::{lamports_to_sol, LAMPORTS_PER_SOL};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::{SeedDerivable, Signer};
use solana_sdk::system_instruction;
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

    let static_wallet = Pubkey::new_unique();

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
    let space = 0u64;
    let rent_exempt = client
        .get_minimum_balance_for_rent_exemption(space as usize)
        .expect("Error get rent exempt");

    // create account instruction
    let create_account_ins = system_instruction::create_account_with_seed(
        &payer,
        &derived_pubkey,
        &payer,
        seed,
        rent_exempt + 2_000_000,
        space,
        &system_program_id,
    );

    // transfer lamports to new created account
    let transfer_to_new_wallet_ins =
        system_instruction::transfer(&payer, &derived_pubkey, rent_exempt + 100_000);

    // transfer to static public key
    let transfer_to_static_wallet_ins =
        system_instruction::transfer(&payer, &static_wallet, rent_exempt + 100_000);

    let (recent_blockhash, _) = client
        .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())
        .expect("Error get recent blockhash");

    let message = Message::new(
        &[
            create_account_ins,
            transfer_to_static_wallet_ins.clone(),
            transfer_to_new_wallet_ins,
            transfer_to_static_wallet_ins,
        ],
        Some(&payer),
    );
    let tx = Transaction::new(&[&payer_keypair], message, recent_blockhash);

    info!("Sending tx... {:#?}", tx);
    client
        .send_and_confirm_transaction(&tx)
        .expect("Error sending tx");
}
