#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

declare_id!("G1n3r3cKW2y3Spgbp7iAdfMeoC8hdsJGBEqg5qRRJSq3");

#[program]
pub mod crowdfunding {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.admin = ctx.accounts.admin.key();
        config.rally_count = 0;
        Ok(())
    }

    pub fn create_rally(
        ctx: Context<CreateRally>,
        title: String,
        description: String,
        goal: u64,
        deadline: i64,
        token_mint: Pubkey
    ) -> Result<()> {
        let rally = &mut ctx.accounts.rally;
        let config = &mut ctx.accounts.config;
        let clock = Clock::get()?;

        require!(clock.unix_timestamp < deadline, ErrorCode::InvalidDeadline);
        require!(goal > 0, ErrorCode::InvalidGoal);
        require!(title.len() <= 100, ErrorCode::InvalidTitle);
        require!(description.len() <= 1000, ErrorCode::InvalidDescription);

        config.rally_count += 1;
        rally.id = config.rally_count;
        rally.creator = ctx.accounts.creator.key();
        rally.title = title;
        rally.description = description;
        rally.goal = goal;
        rally.deadline = deadline;
        rally.raised = 0;
        rally.token_mint = token_mint;
        rally.is_active = true;
        rally.feedback_count = 0;

        Ok(())
    }

    pub fn fund_rally(ctx: Context<FundRally>, amount: u64) -> Result<()> {
        let rally = &mut ctx.accounts.rally;
        let clock = Clock::get()?;

        require!(rally.is_active, ErrorCode::RallyNotActive);
        require!(clock.unix_timestamp <= rally.deadline, ErrorCode::RallyExpired);
        require!(amount > 0, ErrorCode::InvalidAmount);

        // Transfer tokens
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.funder_token.to_account_info(),
            to: ctx.accounts.vault_token.to_account_info(),
            authority: ctx.accounts.funder.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(CpiContext::new(cpi_program, cpi_accounts), amount)?;

        // Update rally
        rally.raised += amount;

        // Update contributor
        let contributor = &mut ctx.accounts.contributor;
        contributor.amount += amount;
        contributor.contribution_count += 1;

        // Update top contributor
        let top_contributor = &mut ctx.accounts.top_contributor;
        top_contributor.amount += amount;
        top_contributor.contribution_count += 1;

        emit!(FundRallyEvent {
            rally_id: rally.id,
            funder: ctx.accounts.funder.key(),
            amount
        });

        Ok(())
    }

    pub fn withdraw_funds(ctx: Context<WithdrawFunds>) -> Result<()> {
        let rally = &mut ctx.accounts.rally;
        
        require!(ctx.accounts.creator.key() == rally.creator, ErrorCode::Unauthorized);
        require!(rally.raised > 0, ErrorCode::NoFunds);

        let amount = rally.raised;
        rally.raised = 0;

        // Transfer tokens from vault to creator
        let seeds = &[
            b"vault".as_ref(),
            rally.to_account_info().key.as_ref(),
            &[ctx.bumps.vault_token],
        ];
        let signer = &[&seeds[..]];
        let cpi_accounts = token::Transfer {
            from: ctx.accounts.vault_token.to_account_info(),
            to: ctx.accounts.creator_token.to_account_info(),
            authority: ctx.accounts.vault_token.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        token::transfer(
            CpiContext::new_with_signer(cpi_program, cpi_accounts, signer),
            amount
        )?;

        emit!(WithdrawFundsEvent {
            rally_id: rally.id,
            amount
        });

        Ok(())
    }

    pub fn add_feedback(ctx: Context<AddFeedback>, message: String) -> Result<()> {
        let rally = &mut ctx.accounts.rally;
        let contributor = &ctx.accounts.contributor;
        let feedback = &mut ctx.accounts.feedback;
        let clock = Clock::get()?;

        require!(rally.is_active, ErrorCode::RallyNotActive);
        require!(contributor.contribution_count > 0, ErrorCode::NotContributor);
        require!(message.len() <= 500, ErrorCode::InvalidFeedback);

        rally.feedback_count += 1;
        feedback.rally_id = rally.id;
        feedback.sender = ctx.accounts.sender.key();
        feedback.message = message;
        feedback.timestamp = clock.unix_timestamp;

        emit!(AddFeedbackEvent {
            rally_id: rally.id,
            feedback_id: rally.feedback_count
        });

        Ok(())
    }

    pub fn close_rally(ctx: Context<CloseRally>) -> Result<()> {
        let rally = &mut ctx.accounts.rally;
        
        require!(ctx.accounts.creator.key() == rally.creator, ErrorCode::Unauthorized);
        
        rally.is_active = false;

        emit!(CloseRallyEvent {
            rally_id: rally.id
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + Config::MAX_SIZE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateRally<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Rally::MAX_SIZE,
        seeds = [b"rally", config.rally_count.to_le_bytes().as_ref()],
        bump
    )]
    pub rally: Account<'info, Rally>,
    #[account(
        mut,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = creator,
        token::mint = token_mint,
        token::authority = vault_token,
        seeds = [b"vault", rally.key().as_ref()],
        bump
    )]
    pub vault_token: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, anchor_spl::token::Mint>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct FundRally<'info> {
    #[account(
        mut,
        seeds = [b"rally", rally.id.to_le_bytes().as_ref()],
        bump
    )]
    pub rally: Account<'info, Rally>,
    #[account(
        mut,
        seeds = [b"vault", rally.key().as_ref()],
        bump
    )]
    pub vault_token: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = funder,
        space = 8 + Contributor::MAX_SIZE,
        seeds = [b"contributor", rally.key().as_ref(), funder.key().as_ref()],
        bump
    )]
    pub contributor: Account<'info, Contributor>,
    #[account(
        init_if_needed,
        payer = funder,
        space = 8 + TopContributor::MAX_SIZE,
        seeds = [b"top_contributor", funder.key().as_ref()],
        bump
    )]
    pub top_contributor: Account<'info, TopContributor>,
    #[account(mut)]
    pub funder_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub funder: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(
        mut,
        seeds = [b"rally", rally.id.to_le_bytes().as_ref()],
        bump
    )]
    pub rally: Account<'info, Rally>,
    #[account(
        mut,
        seeds = [b"vault", rally.key().as_ref()],
        bump
    )]
    pub vault_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub creator_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AddFeedback<'info> {
    #[account(
        mut,
        seeds = [b"rally", rally.id.to_le_bytes().as_ref()],
        bump
    )]
    pub rally: Account<'info, Rally>,
    #[account(
        init,
        payer = sender,
        space = 8 + Feedback::MAX_SIZE,
        seeds = [b"feedback", rally.key().as_ref(), rally.feedback_count.to_le_bytes().as_ref()],
        bump
    )]
    pub feedback: Account<'info, Feedback>,
    #[account(
        seeds = [b"contributor", rally.key().as_ref(), sender.key().as_ref()],
        bump
    )]
    pub contributor: Account<'info, Contributor>,
    #[account(mut)]
    pub sender: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseRally<'info> {
    #[account(
        mut,
        seeds = [b"rally", rally.id.to_le_bytes().as_ref()],
        bump
    )]
    pub rally: Account<'info, Rally>,
    #[account(mut)]
    pub creator: Signer<'info>,
}

#[account]
#[derive(Default)]
pub struct Config {
    pub admin: Pubkey,
    pub rally_count: u64,
}

#[account]
#[derive(Default)]
pub struct Rally {
    pub id: u64,
    pub creator: Pubkey,
    pub title: String,
    pub description: String,
    pub goal: u64,
    pub deadline: i64,
    pub raised: u64,
    pub token_mint: Pubkey,
    pub is_active: bool,
    pub feedback_count: u64,
}

#[account]
#[derive(Default)]
pub struct Feedback {
    pub rally_id: u64,
    pub sender: Pubkey,
    pub message: String,
    pub timestamp: i64,
}

#[account]
#[derive(Default)]
pub struct Contributor {
    pub amount: u64,
    pub contribution_count: u64,
}

#[account]
#[derive(Default)]
pub struct TopContributor {
    pub amount: u64,
    pub contribution_count: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized action")]
    Unauthorized,
    #[msg("Rally is not active")]
    RallyNotActive,
    #[msg("Rally has expired")]
    RallyExpired,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("No funds to withdraw")]
    NoFunds,
    #[msg("Only contributors can add feedback")]
    NotContributor,
    #[msg("Invalid deadline")]
    InvalidDeadline,
    #[msg("Invalid goal")]
    InvalidGoal,
    #[msg("Title too long")]
    InvalidTitle,
    #[msg("Description too long")]
    InvalidDescription,
    #[msg("Feedback message too long")]
    InvalidFeedback,
}

#[event]
pub struct FundRallyEvent {
    pub rally_id: u64,
    pub funder: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WithdrawFundsEvent {
    pub rally_id: u64,
    pub amount: u64,
}

#[event]
pub struct AddFeedbackEvent {
    pub rally_id: u64,
    pub feedback_id: u64,
}

#[event]
pub struct CloseRallyEvent {
    pub rally_id: u64,
}

impl Config {
    pub const MAX_SIZE: usize = 32 + 8; // Pubkey + u64
}

impl Rally {
    pub const MAX_SIZE: usize = 8 + 32 + 4 + 100 + 4 + 1000 + 8 + 8 + 8 + 32 + 1 + 8; // All fields
}

impl Feedback {
    pub const MAX_SIZE: usize = 8 + 32 + 4 + 500 + 8; // All fields
}

impl Contributor {
    pub const MAX_SIZE: usize = 8 + 8; // u64 + u64
}

impl TopContributor {
    pub const MAX_SIZE: usize = 8 + 8; // u64 + u64
}