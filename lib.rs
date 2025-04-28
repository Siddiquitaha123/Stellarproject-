#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, Address, symbol_short};

// Faucet settings structure
#[contracttype]
#[derive(Clone)]
pub struct FaucetConfig {
    pub admin: Address,
    pub token_address: Address,
    pub distribution_amount: u64,
    pub cooldown_period: u64,
    pub total_distributed: u64,
    pub distribution_count: u64,
    pub is_active: bool
}

// Structure to track user claims
#[contracttype]
#[derive(Clone)]
pub struct UserClaim {
    pub user: Address,
    pub last_claim_timestamp: u64,
    pub total_claims: u64,
    pub total_amount: u64
}

// Mapping user address to their claim record
#[contracttype]
pub enum UserClaimRegistry {
    User(Address)
}

// Key for storing faucet configuration
const FAUCET_CONFIG: Symbol = symbol_short!("CONFIG");

#[contract]
pub struct TokenFaucet;

#[contractimpl]
impl TokenFaucet {
    // Initialize the token faucet
    pub fn initialize(
        env: Env,
        admin: Address,
        token_address: Address,
        distribution_amount: u64,
        cooldown_period: u64
    ) {
        // Check if already initialized
        if env.storage().instance().has(&FAUCET_CONFIG) {
            log!(&env, "Faucet already initialized");
            panic!("Faucet already initialized");
        }
        
        // Create faucet configuration
        let config = FaucetConfig {
            admin: admin.clone(),
            token_address,
            distribution_amount,
            cooldown_period,
            total_distributed: 0,
            distribution_count: 0,
            is_active: true
        };
        
        // Store configuration
        env.storage().instance().set(&FAUCET_CONFIG, &config);
        env.storage().instance().extend_ttl(10000, 10000);
        
        log!(&env, "Faucet initialized with admin");
    }
    
    // Request tokens from the faucet
    pub fn request_tokens(env: Env, user: Address) -> u64 {
        // Authenticate user
        user.require_auth();
        
        // Get faucet configuration
        let config: FaucetConfig = env.storage().instance().get(&FAUCET_CONFIG)
            .expect("Faucet not initialized");
            
        // Check if faucet is active
        if !config.is_active {
            log!(&env, "Faucet is currently inactive");
            panic!("Faucet is currently inactive");
        }
        
        // Get user's claim history
        let user_key = UserClaimRegistry::User(user.clone());
        let mut user_claim: UserClaim = env.storage().instance().get(&user_key).unwrap_or(
            UserClaim {
                user: user.clone(),
                last_claim_timestamp: 0,
                total_claims: 0,
                total_amount: 0
            }
        );
        
        // Get current timestamp
        let current_time = env.ledger().timestamp();
        
        // Check cooldown period
        if user_claim.last_claim_timestamp > 0 && 
           (current_time - user_claim.last_claim_timestamp) < config.cooldown_period {
            log!(&env, "Cooldown period not elapsed yet");
            panic!("Cooldown period not elapsed yet");
        }
        
        // Update user claim records
        user_claim.last_claim_timestamp = current_time;
        user_claim.total_claims += 1;
        user_claim.total_amount += config.distribution_amount;
        
        // Update faucet stats
        let mut updated_config = config.clone();
        updated_config.total_distributed += config.distribution_amount;
        updated_config.distribution_count += 1;
        
        // Store updated records
        env.storage().instance().set(&user_key, &user_claim);
        env.storage().instance().set(&FAUCET_CONFIG, &updated_config);
        env.storage().instance().extend_ttl(10000, 10000);
        
        log!(&env, "Tokens distributed to user");
        
        // Return the distribution amount
        config.distribution_amount
    }
    
    // Update faucet configuration (admin only)
    pub fn update_config(
        env: Env,
        admin: Address,
        new_distribution_amount: Option<u64>,
        new_cooldown_period: Option<u64>,
        new_active_status: Option<bool>
    ) {
        // Authenticate admin
        admin.require_auth();
        
        // Get current config
        let mut config: FaucetConfig = env.storage().instance().get(&FAUCET_CONFIG)
            .expect("Faucet not initialized");
            
        // Verify admin
        if config.admin != admin {
            log!(&env, "Only admin can update configuration");
            panic!("Only admin can update configuration");
        }
        
        // Update configuration if provided
        if let Some(amount) = new_distribution_amount {
            config.distribution_amount = amount;
        }
        
        if let Some(period) = new_cooldown_period {
            config.cooldown_period = period;
        }
        
        if let Some(status) = new_active_status {
            config.is_active = status;
        }
        
        // Store updated configuration
        env.storage().instance().set(&FAUCET_CONFIG, &config);
        env.storage().instance().extend_ttl(10000, 10000);
        
        log!(&env, "Faucet configuration updated");
    }
    
    // Get faucet configuration
    pub fn get_config(env: Env) -> FaucetConfig {
        env.storage().instance().get(&FAUCET_CONFIG)
            .expect("Faucet not initialized")
    }
    
    // Get user claim history
    pub fn get_user_claims(env: Env, user: Address) -> UserClaim {
        let user_key = UserClaimRegistry::User(user.clone());
        env.storage().instance().get(&user_key).unwrap_or(
            UserClaim {
                user: user.clone(),
                last_claim_timestamp: 0,
                total_claims: 0,
                total_amount: 0
            }
        )
    }
}