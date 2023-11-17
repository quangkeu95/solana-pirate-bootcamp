use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::message::Message;
use solana_sdk::native_token::{lamports_to_sol, LAMPORTS_PER_SOL};
use solana_sdk::program_pack::Pack;
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
    let seed = "token_001";

    // let mint_pubkey = Pubkey::create_with_seed(&payer, seed, &system_program_id)
    //     .expect("Error creating mint pubkey");
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    info!("Mint program id {}", mint_pubkey);

    // get rent exempt
    let mint_space = spl_token::state::Mint::LEN;
    let mint_rent_exempt = client
        .get_minimum_balance_for_rent_exemption(mint_space as usize)
        .expect("Error get mint rent exempt");
    let create_mint_account_ins = system_instruction::create_account(
        &payer,
        &mint_pubkey,
        mint_rent_exempt,
        mint_space as u64,
        &spl_token::id(),
    );

    // init mint
    let init_mint_ins = spl_token::instruction::initialize_mint2(
        &spl_token::id(),
        &mint_pubkey,
        &payer,
        Some(&payer),
        2,
    )
    .expect("Error init mint2");

    // create metadata
    let (metadata_pubkey, bump_seed) = Pubkey::find_program_address(
        &[
            b"metadata",
            &mpl_token_metadata::ID.to_bytes(),
            &mint_pubkey.to_bytes(),
        ],
        &mpl_token_metadata::ID,
    );

    info!("Metadata address {}", metadata_pubkey);

    let create_metadata_ins =
        mpl_token_metadata::instructions::CreateMetadataAccountV3Builder::new()
            .metadata(metadata_pubkey)
            .mint(mint_pubkey)
            .mint_authority(payer)
            .payer(payer)
            .update_authority(payer, true)
            .data(mpl_token_metadata::types::DataV2 {
                name: "Seven Seas Gold".to_string(),
                symbol: "GOLD".to_string(),
                uri: "https://thisisnot.arealurl/info.json".to_string(),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            })
            .is_mutable(true)
            .instruction();

    // create associate token account instruction
    let ata_pubkey = spl_associated_token_account::get_associated_token_address_with_program_id(
        &payer,
        &mint_pubkey,
        &spl_token::id(),
    );
    let create_ata_ins = spl_associated_token_account::instruction::create_associated_token_account(
        &payer,
        &payer,
        &mint_pubkey,
        &spl_token::id(),
    );

    // create mint instruction
    let mint_ins = spl_token::instruction::mint_to_checked(
        &spl_token::id(),
        &mint_pubkey,
        &ata_pubkey,
        &payer,
        &[&payer],
        100,
        2,
    )
    .expect("Error create mint instruction");

    let message = Message::new(
        &[
            create_mint_account_ins,
            init_mint_ins,
            create_metadata_ins,
            create_ata_ins,
            mint_ins,
        ],
        Some(&payer),
    );

    let (recent_blockhash, _) = client
        .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())
        .expect("Error get recent blockhash");
    let tx = Transaction::new(
        &[&payer_keypair, &mint_keypair, &payer_keypair],
        message,
        recent_blockhash,
    );

    info!("Sending tx... {:#?}", tx);
    client
        .send_and_confirm_transaction(&tx)
        .map_err(|e| {
            error!("{:#?}", e);
        })
        .unwrap();

    // get mint account info
    let mint_account = client
        .get_account(&mint_pubkey)
        .expect("Error get mint account");
    info!("Mint account {:#?}", mint_account);

    let ata_balance = client
        .get_token_account_balance(&ata_pubkey)
        .expect("Error get ata balance");
    info!("ATA balance = {:#?}", ata_balance);
    assert_eq!(ata_balance.amount, "100");
}
