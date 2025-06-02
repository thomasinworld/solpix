use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkgMQoezjZ8h1");

// CHANGE THIS TO YOUR WALLET!
const PROJECT_WALLET: &str = "YOUR_PROJECT_WALLET_PUBKEY";

#[program]
pub mod solpix {
    use super::*;

    pub fn register_username(ctx: Context<RegisterUsername>, username: String) -> Result<()> {
        require!(username.len() > 2 && username.len() < 32, ErrorCode::InvalidUsernameLength);
        let user_profile = &mut ctx.accounts.user_profile;
        let username_registry = &mut ctx.accounts.username_registry;

        require!(!username_registry.is_taken, ErrorCode::UsernameTaken);

        user_profile.authority = ctx.accounts.authority.key();
        user_profile.username = username.clone();
        user_profile.post_count = 0;

        username_registry.username = username;
        username_registry.owner = ctx.accounts.authority.key();
        username_registry.is_taken = true;

        // Fee to project wallet
        let fee = 1_000_000; // 0.001 SOL in lamports
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &ctx.accounts.project_wallet.key(),
            fee,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.project_wallet.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn create_post(
        ctx: Context<CreatePost>,
        nft_mint: Pubkey,
        caption: String,
    ) -> Result<()> {
        require!(caption.len() < 256, ErrorCode::CaptionTooLong);

        let post = &mut ctx.accounts.post;
        let user_profile = &mut ctx.accounts.user_profile;

        post.owner = ctx.accounts.authority.key();
        post.nft_mint = nft_mint;
        post.caption = caption;
        post.timestamp = Clock::get()?.unix_timestamp;

        user_profile.post_count += 1;

        // Fee to project wallet
        let fee = 1_000_000; // 0.001 SOL in lamports
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &ctx.accounts.project_wallet.key(),
            fee,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.project_wallet.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn list_nft(ctx: Context<ListNFT>, price: u64) -> Result<()> {
        let listing = &mut ctx.accounts.listing;
        listing.seller = ctx.accounts.authority.key();
        listing.nft_mint = ctx.accounts.nft_mint.key();
        listing.price = price;
        listing.active = true;

        // Fee to project wallet
        let fee = 1_000_000; // 0.001 SOL in lamports
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.authority.key(),
            &ctx.accounts.project_wallet.key(),
            fee,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.authority.to_account_info(),
                ctx.accounts.project_wallet.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn buy_nft(ctx: Context<BuyNFT>) -> Result<()> {
        let listing = &mut ctx.accounts.listing;
        require!(listing.active, ErrorCode::ListingInactive);

        // Buyer pays seller minus fee
        let fee = 1_000_000; // 0.001 SOL in lamports
        let seller_amount = listing.price.checked_sub(fee).ok_or(ErrorCode::InvalidPrice)?;

        // Transfer SOL to seller
        let ix1 = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &ctx.accounts.seller.key(),
            seller_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix1,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.seller.to_account_info(),
            ],
        )?;

        // Transfer fee to project wallet
        let ix2 = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &ctx.accounts.project_wallet.key(),
            fee,
        );
        anchor_lang::solana_program::program::invoke(
            &ix2,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.project_wallet.to_account_info(),
            ],
        )?;

        // Mark listing inactive
        listing.active = false;

        // NFT transfer should be handled in frontend using Metaplex or Token Program

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(username: String)]
pub struct RegisterUsername<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 36 + 8,
        seeds = [b"user_profile", authority.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    #[account(
        init,
        payer = authority,
        space = 8 + 36 + 32 + 1,
        seeds = [b"username_registry", username.as_bytes()],
        bump
    )]
    pub username_registry: Account<'info, UsernameRegistry>,

    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is safe, project wallet is a trusted address
    #[account(mut, address = Pubkey::from_str(PROJECT_WALLET).unwrap())]
    pub project_wallet: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreatePost<'info> {
    #[account(
        mut,
        seeds = [b"user_profile", authority.key().as_ref()],
        bump
    )]
    pub user_profile: Account<'info, UserProfile>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 256 + 8,
        seeds = [b"post", authority.key().as_ref(), &[user_profile.post_count as u8]],
        bump
    )]
    pub post: Account<'info, Post>,

    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is safe, project wallet is a trusted address
    #[account(mut, address = Pubkey::from_str(PROJECT_WALLET).unwrap())]
    pub project_wallet: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ListNFT<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 8 + 1,
        seeds = [b"listing", nft_mint.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, Listing>,

    pub nft_mint: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is safe, project wallet is a trusted address
    #[account(mut, address = Pubkey::from_str(PROJECT_WALLET).unwrap())]
    pub project_wallet: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyNFT<'info> {
    #[account(
        mut,
        seeds = [b"listing", listing.nft_mint.as_ref()],
        bump
    )]
    pub listing: Account<'info, Listing>,

    #[account(mut)]
    pub buyer: Signer<'info>,
    /// CHECK: This is safe, seller is a trusted address
    #[account(mut)]
    pub seller: UncheckedAccount<'info>,
    /// CHECK: This is safe, project wallet is a trusted address
    #[account(mut, address = Pubkey::from_str(PROJECT_WALLET).unwrap())]
    pub project_wallet: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct UserProfile {
    pub authority: Pubkey,
    pub username: String,
    pub post_count: u64,
}

#[account]
pub struct UsernameRegistry {
    pub username: String,
    pub owner: Pubkey,
    pub is_taken: bool,
}

#[account]
pub struct Post {
    pub owner: Pubkey,
    pub nft_mint: Pubkey,
    pub caption: String,
    pub timestamp: i64,
}

#[account]
pub struct Listing {
    pub seller: Pubkey,
    pub nft_mint: Pubkey,
    pub price: u64,
    pub active: bool,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Username must be between 3 and 31 characters.")]
    InvalidUsernameLength,
    #[msg("Username is already taken.")]
    UsernameTaken,
    #[msg("Caption too long.")]
    CaptionTooLong,
    #[msg("Listing is not active.")]
    ListingInactive,
    #[msg("Invalid price.")]
    InvalidPrice,
} 