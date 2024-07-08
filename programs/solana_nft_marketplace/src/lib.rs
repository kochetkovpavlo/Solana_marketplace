use anchor_lang::prelude::*;
use anchor_spl::token::{Token,TokenAccount,Transfer};

declare_id!("HnEci36EQ4njjc1DK7XjEyykubLVXEzSux3KR1ga31wB");

#[program]
pub mod nft_marketplace {
    use super::*;

    pub fn list_nft(ctx: Context<ListNft>, price: u64) -> ProgramResult {
        let listing = &mut ctx.accounts.listing;
        listing.owner = *ctx.accounts.owner.key;
        listing.mint = *ctx.accounts.mint.to_account_info().key;
        listing.price = price;
        listing.is_active = true;
        Ok(())
    }

    pub fn cancel_listing(ctx: Context<CancelListing>) -> ProgramResult {
        let listing = &mut ctx.accounts.listing;
        require!(listing.owner == *ctx.accounts.owner.key, ErrorCode::Unauthorized);
        listing.is_active = false;
        Ok(())
    }

    pub fn buy_nft(ctx: Context<BuyNft>) -> ProgramResult {
        let listing = &mut ctx.accounts.listing;

        // Ensure listing is active
        require!(listing.is_active, ErrorCode::InactiveListing);

        // Ensure buyer sent enough SOL to buy the NFT
        if **ctx.accounts.buyer.to_account_info().lamports.borrow() < listing.price {
            return Err(ErrorCode::InsufficientFunds.into());
        }

        // Transfer SOL from buyer to seller
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.buyer.key,
            &ctx.accounts.seller.key,
            listing.price,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.seller.to_account_info(),
            ],
        )?;

        // Transfer NFT from seller to buyer
        let cpi_accounts = Transfer {
            from: ctx.accounts.seller_token_account.to_account_info().clone(),
            to: ctx.accounts.buyer_token_account.to_account_info().clone(),
            authority: ctx.accounts.seller.clone(),
        };
        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        anchor_spl::token::transfer(cpi_ctx, 1)?;

        listing.is_active = false;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct ListNft<'info> {
    #[account(init, payer = owner, space = 8 + 32 + 32 + 8 + 1)]
    pub listing: Account<'info, NftListing>,
    #[account(signer)]
    pub owner: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelListing<'info> {
    #[account(mut, has_one = owner)]
    pub listing: Account<'info, NftListing>,
    #[account(signer)]
    pub owner: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct BuyNft<'info> {
    #[account(mut)]
    pub listing: Account<'info, NftListing>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub seller: AccountInfo<'info>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct NftListing {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub price: u64,
    pub is_active: bool,
}

#[error]
pub enum ErrorCode {
    #[msg("Insufficient funds to buy NFT.")]
    InsufficientFunds,
    #[msg("Listing is not active.")]
    InactiveListing,
    #[msg("Unauthorized action.")]
    Unauthorized,
}