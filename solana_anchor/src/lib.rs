use anchor_lang::{
    prelude::*,
    Discriminator,
};
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_anchor {
    use super::*;

    pub fn init_pool(
        ctx : Context<InitPool>,
        _bump : u8,
        _start_at : u64,
        _limit : u64,
        _reward_lock_period : u64,
        _withdraw_lock_period : u64,
        _fee : u64,
        ) -> ProgramResult {
        msg!("+ init_pool");

        let pool = &mut ctx.accounts.pool;

        pool.owner = *ctx.accounts.owner.key;
        pool.rand = *ctx.accounts.rand.key;
        pool.token_mint = ctx.accounts.token_mint.key();
        pool.collection = *ctx.accounts.collection.key;
        pool.fee_receiver = ctx.accounts.fee_receiver.key();
        pool.trader = ctx.accounts.trader.key();
        pool.start_at = _start_at;
        pool.deposit_limit = _limit;
        pool.reward_lock_period = _reward_lock_period;
        pool.withdraw_lock_period = _withdraw_lock_period;
        pool.fee = _fee;
        pool.tvl = 0;
        pool.reward = 0;
        pool.decimals = ctx.accounts.token_mint.decimals;
        pool.bump = _bump;
        pool.pool_ledger = *ctx.accounts.pool_ledger.key;

        let mut data = (&mut ctx.accounts.pool_ledger).data.borrow_mut();
        let mut new_data = RateList::discriminator().try_to_vec().unwrap();
        new_data.append(&mut pool.key().try_to_vec().unwrap());
        new_data.append(&mut (0 as u8).try_to_vec().unwrap());
        new_data.append(&mut (0 as u32).try_to_vec().unwrap());
        for i in 0..new_data.len(){
            data[i] = new_data[i];
        }
        let vec_start = 8 + 32 + 1 + 4;
        let as_bytes = (MAX_LEN as u32).to_le_bytes();
        for i in 0..4{
            data[vec_start+i] = as_bytes[i];
        }

        Ok(())
    }

    pub fn update_pool(
        ctx : Context<UpdatePool>,
        _limit : u64,
        _reward_lock_period : u64,
        _withdraw_lock_period : u64,
        _fee : u64,
        ) -> ProgramResult {

        msg!("Update");

        let pool = &mut ctx.accounts.pool;

        pool.deposit_limit = _limit;
        pool.reward_lock_period = _reward_lock_period;
        pool.withdraw_lock_period = _withdraw_lock_period;
        pool.fee = _fee;

        pool.owner = *ctx.accounts.new_owner.key;

        Ok(())
    }

    pub fn change_ledger(
        ctx : Context<ChangeLedger>
        ) -> ProgramResult {

        msg!("ChangeLedger");

        let pool = &mut ctx.accounts.pool;

        pool.pool_ledger = *ctx.accounts.pool_ledger.key;

        Ok(())
    }

    pub fn init_user(
        ctx : Context<InitUser>,
        _bump : u8,
        _ranking : u32
        ) -> ProgramResult {
        msg!("+ init_user");
        
        let user_data = &mut ctx.accounts.user_data;
        let ledger = get_reward_rate(&ctx.accounts.pool_ledger, _ranking as usize)?;

        if ctx.accounts.nft_mint.key() != ledger.mint {
            return Err(PoolError::InvalidRanking.into());
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_nft_account.to_account_info().clone(),
            to: ctx.accounts.pool_nft_account.to_account_info().clone(),
            authority: ctx.accounts.owner.to_account_info().clone(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info().clone();

        let token_cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(token_cpi_ctx, 1)?;

        user_data.owner = *ctx.accounts.owner.key;
        user_data.pool = ctx.accounts.pool.key();
        user_data.nft_mint = ctx.accounts.nft_mint.key();
        user_data.stake_amount = 0;
        user_data.reward_rate = ledger.reward_rate;
        user_data.last_time = 0;
        user_data.withdraw_amount = 0;
        user_data.request_time = 0;
        user_data.withdrawable = false;
        user_data.bump = _bump;

        Ok(())
    }

    pub fn update_user(
        ctx : Context<UpdateUser>,
        _ranking : u32
        ) -> ProgramResult {
        msg!("+ update_user");
        
        let user_data = &mut ctx.accounts.user_data;
        let ledger = get_reward_rate(&ctx.accounts.pool_ledger, _ranking as usize)?;

        if ctx.accounts.nft_mint.key() != ledger.mint {
            return Err(PoolError::InvalidRanking.into());
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_nft_account.to_account_info().clone(),
            to: ctx.accounts.pool_nft_account.to_account_info().clone(),
            authority: ctx.accounts.owner.to_account_info().clone(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info().clone();

        let token_cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(token_cpi_ctx, 1)?;

        user_data.owner = *ctx.accounts.owner.key;
        user_data.pool = ctx.accounts.pool.key();
        user_data.nft_mint = ctx.accounts.nft_mint.key();
        user_data.stake_amount = 0;
        user_data.reward_rate = ledger.reward_rate;
        user_data.last_time = 0;
        user_data.withdraw_amount = 0;
        user_data.request_time = 0;
        user_data.withdrawable = false;

        Ok(())
    }

    pub fn set_rate(
        ctx : Context<SetRate>,
        _rasing_rate : u8
        ) ->ProgramResult {
        msg!("+ set_rate");

        let pool_ledger = &mut ctx.accounts.pool_ledger;

        set_rasing_rate(pool_ledger, _rasing_rate);

        Ok(())
    }

    pub fn set_list(
        ctx : Context<SetList>,
        nfts : Vec<Pubkey>
        ) ->ProgramResult {
        msg!("+ set_list");

        let rasing_rate = get_rasing_rate(&ctx.accounts.pool_ledger)?;
        let last_number = get_last_number(&ctx.accounts.pool_ledger)?;

        if last_number == 0 {
            for i in 0..nfts.len(){
                set_reward_rate(&mut ctx.accounts.pool_ledger, (last_number + i as u32) as usize, RewardRate{
                    reward_rate : 1000 + rasing_rate as u64 * i as u64,
                    mint : nfts[i],
                });
            }
        } else {
            let last_ledger = get_reward_rate(&ctx.accounts.pool_ledger, last_number as usize - 1)?;
    
            for i in 0..nfts.len(){
                set_reward_rate(&mut ctx.accounts.pool_ledger, (last_number + i as u32) as usize, RewardRate{
                    reward_rate : last_ledger.reward_rate + rasing_rate as u64 * (i as u64 + 1),
                    mint : nfts[i],
                });
            }
        }

        set_last_number(&mut ctx.accounts.pool_ledger, last_number + nfts.len() as u32);

        Ok(())
    }

    pub fn deposit(
        ctx : Context<Deposit>,
        _amount : u64,
        ) -> ProgramResult {
        msg!("+ stake");

        let pool = &mut ctx.accounts.pool;
        let user_data = &mut ctx.accounts.user_data;

        let mut limit = pool.deposit_limit;
        if pool.deposit_limit != 1000_000_000_000 && user_data.reward_rate >= 10 && user_data.reward_rate < 20 {
            limit = pool.deposit_limit;
        }
        if pool.deposit_limit != 1000_000_000_000 && user_data.reward_rate >= 20 && user_data.reward_rate < 30 {
            limit = pool.deposit_limit * 3 / 4;
        }
        if pool.deposit_limit != 1000_000_000_000 && user_data.reward_rate >= 30 && user_data.reward_rate < 40 {
            limit = pool.deposit_limit / 2;
        }

        if user_data.stake_amount + _amount > limit {
            return Err(PoolError::InvalidStakeAmount.into());
        }

        let token_cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info().clone(),
            to: ctx.accounts.pool_token_account.to_account_info().clone(),
            authority: ctx.accounts.owner.to_account_info().clone(),
        };
        
        let token_cpi_program = ctx.accounts.token_program.to_account_info().clone();
        let token_cpi_ctx = CpiContext::new(token_cpi_program, token_cpi_accounts);
        
        token::transfer(token_cpi_ctx, _amount * 4 /10)?;
        
        let clock = Clock::from_account_info(&ctx.accounts.clock)?;

        let trade_cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info().clone(),
            to: ctx.accounts.trader.to_account_info().clone(),
            authority: ctx.accounts.owner.to_account_info().clone(),
        };
        
        let trade_cpi_program = ctx.accounts.token_program.to_account_info().clone();
        let trade_cpi_ctx = CpiContext::new(trade_cpi_program, trade_cpi_accounts);
        
        token::transfer(trade_cpi_ctx, _amount * 6 /10)?;
        
        let clock = Clock::from_account_info(&ctx.accounts.clock)?;

        if (user_data.stake_amount - user_data.withdraw_amount) > 0 && (clock.unix_timestamp as u64 - user_data.last_time) > pool.reward_lock_period * PERIOD as u64 {
            let total_reward_amount = (user_data.stake_amount - user_data.withdraw_amount) * user_data.reward_rate / 10000 / 365 / 24 / 60 / 60 * (clock.unix_timestamp as u64 - user_data.last_time);

            let reward_cpi_accounts = Transfer {
                from: ctx.accounts.pool_token_account.to_account_info().clone(),
                to: ctx.accounts.user_token_account.to_account_info().clone(),
                authority: pool.to_account_info().clone(),
            };
    
            let reward_cpi_program = ctx.accounts.token_program.to_account_info().clone();
    
            let pool_signer_seeds = &[
                pool.rand.as_ref(),
                &[pool.bump],
            ];
    
            let pool_signer = &[&pool_signer_seeds[..]];
    
            let reward_cpi_ctx = CpiContext::new_with_signer(reward_cpi_program, reward_cpi_accounts, pool_signer);
    
            token::transfer(reward_cpi_ctx, total_reward_amount)?;

            pool.reward += total_reward_amount;
        }

        user_data.stake_amount += _amount;
        user_data.last_time = clock.unix_timestamp as u64;

        pool.tvl += _amount;
        
        Ok(())
    }

    pub fn emergency_withdraw(
        ctx : Context<EmergencyWithdraw>,
        _amount : u64
        ) -> ProgramResult {
        msg!("+withdraw");

        let pool = &mut ctx.accounts.pool;

        let token_cpi_accounts = Transfer {
            from: ctx.accounts.pool_token_account.to_account_info().clone(),
            to: ctx.accounts.user_token_account.to_account_info().clone(),
            authority: pool.to_account_info().clone(),
        };

        let token_cpi_program = ctx.accounts.token_program.to_account_info().clone();

        let pool_signer_seeds = &[
            pool.rand.as_ref(),
            &[pool.bump],
        ];

        let pool_signer = &[&pool_signer_seeds[..]];

        let token_cpi_ctx = CpiContext::new_with_signer(token_cpi_program, token_cpi_accounts, pool_signer);

        token::transfer(token_cpi_ctx, _amount)?;

        Ok(())
    }

    pub fn instant_withdraw(
        ctx : Context<InstantWithdraw>,
        _amount:u64
        ) -> ProgramResult {
        msg!("+ instant withdraw");

        let user_data = &mut ctx.accounts.user_data;
        let pool = &mut ctx.accounts.pool;

        let clock = Clock::from_account_info(&ctx.accounts.clock)?;
        
        if user_data.stake_amount == 0{
            return Err(PoolError::InvalidStakeAmount.into());
        }
        
        if user_data.stake_amount != 0 && (clock.unix_timestamp as u64 - user_data.last_time) > pool.reward_lock_period * PERIOD as u64 {
            let total_reward_amount = user_data.stake_amount * user_data.reward_rate / 10000 / 365 / 24 / 60 / 60 * (clock.unix_timestamp as u64 - user_data.last_time);

            let reward_cpi_accounts = Transfer {
                from: ctx.accounts.pool_token_account.to_account_info().clone(),
                to: ctx.accounts.user_token_account.to_account_info().clone(),
                authority: pool.to_account_info().clone(),
            };

            let pool_signer_seeds = &[
                pool.rand.as_ref(),
                &[pool.bump],
            ];

            let pool_signer = &[&pool_signer_seeds[..]];
    
            let reward_cpi_program = ctx.accounts.token_program.to_account_info().clone();
    
            let reward_cpi_ctx = CpiContext::new_with_signer(reward_cpi_program, reward_cpi_accounts, pool_signer);
    
            token::transfer(reward_cpi_ctx, total_reward_amount)?;

            pool.reward += total_reward_amount;
        }

        let pool_signer_seeds = &[
            pool.rand.as_ref(),
            &[pool.bump],
        ];

        let pool_signer = &[&pool_signer_seeds[..]];
        
        let mut amount = _amount;
        if amount >= user_data.stake_amount {
            amount = user_data.stake_amount;
        }

        if amount == user_data.stake_amount {
            let cpi_accounts = Transfer {
                from: ctx.accounts.pool_nft_account.to_account_info().clone(),
                to: ctx.accounts.user_nft_account.to_account_info().clone(),
                authority: pool.to_account_info().clone(),
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info().clone();
    
            let token_cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, pool_signer);
    
            token::transfer(token_cpi_ctx, 1)?;
        }

        let fee_amount = amount * pool.fee / 10000;
        let withdraw_amount = amount - fee_amount;

        let token_cpi_accounts = Transfer {
            from: ctx.accounts.pool_token_account.to_account_info().clone(),
            to: ctx.accounts.user_token_account.to_account_info().clone(),
            authority: pool.to_account_info().clone(),
        };

        let token_cpi_program = ctx.accounts.token_program.to_account_info().clone();

        let token_cpi_ctx = CpiContext::new_with_signer(token_cpi_program, token_cpi_accounts, pool_signer);

        token::transfer(token_cpi_ctx, withdraw_amount)?;

        let fee_cpi_accounts = Transfer {
            from: ctx.accounts.pool_token_account.to_account_info().clone(),
            to: ctx.accounts.fee_receiver.to_account_info().clone(),
            authority: pool.to_account_info().clone(),
        };
        
        let fee_cpi_program = ctx.accounts.token_program.to_account_info().clone();

        let fee_cpi_ctx = CpiContext::new_with_signer(fee_cpi_program, fee_cpi_accounts, pool_signer);
        
        token::transfer(fee_cpi_ctx, fee_amount)?;

        user_data.stake_amount -= amount;
        if user_data.withdrawable && (user_data.stake_amount - user_data.withdraw_amount) < amount {
            user_data.withdraw_amount -= amount - (user_data.stake_amount - user_data.withdraw_amount);
        }

        pool.tvl -= amount;

        user_data.last_time = clock.unix_timestamp as u64;

        if user_data.stake_amount ==0 {
            user_data.withdrawable = false;
            user_data.withdraw_amount = 0;
        }

        Ok(())
    }

    pub fn request_withdraw(
        ctx : Context<RequestWithdraw>,
        _amount : u64
        ) -> ProgramResult {
        msg!("+ request withdraw");

        let user_data = &mut ctx.accounts.user_data;
        let pool = &mut ctx.accounts.pool;

        let clock = Clock::from_account_info(&ctx.accounts.clock)?;
        
        if user_data.stake_amount == 0{
            return Err(PoolError::InvalidStakeAmount.into());
        }

        let pool_signer_seeds = &[
            pool.rand.as_ref(),
            &[pool.bump],
        ];

        let pool_signer = &[&pool_signer_seeds[..]];
        
        if user_data.stake_amount != 0 && (clock.unix_timestamp as u64 - user_data.last_time) > pool.reward_lock_period * PERIOD as u64 {
            let total_reward_amount = user_data.stake_amount * user_data.reward_rate / 10000 / 365 / 24 / 60 / 60 * (clock.unix_timestamp as u64 - user_data.last_time);

            let reward_cpi_accounts = Transfer {
                from: ctx.accounts.pool_token_account.to_account_info().clone(),
                to: ctx.accounts.user_token_account.to_account_info().clone(),
                authority: pool.to_account_info().clone(),
            };
    
            let reward_cpi_program = ctx.accounts.token_program.to_account_info().clone();
    
            let reward_cpi_ctx = CpiContext::new_with_signer(reward_cpi_program, reward_cpi_accounts, pool_signer);
    
            token::transfer(reward_cpi_ctx, total_reward_amount)?;

            pool.reward += total_reward_amount;
        }

        let mut amount = _amount;

        if _amount > user_data.stake_amount{
            amount = user_data.stake_amount;
        }

        user_data.last_time = clock.unix_timestamp as u64;
        user_data.withdraw_amount = amount;
        user_data.withdrawable = true;
        user_data.request_time = clock.unix_timestamp as u64;

        Ok(())
    }

    pub fn cancel_request(
        ctx : Context<CancelRequest>
        ) -> ProgramResult {
        msg!("+ cancel request");

        let user_data = &mut ctx.accounts.user_data;
        let pool = &mut ctx.accounts.pool;

        let clock = Clock::from_account_info(&ctx.accounts.clock)?;
        
        if user_data.stake_amount == 0{
            return Err(PoolError::InvalidStakeAmount.into());
        }
        if user_data.withdrawable == false {
            return Err(PoolError::InvalidStakeAmount.into());
        }

        let pool_signer_seeds = &[
            pool.rand.as_ref(),
            &[pool.bump],
        ];

        let pool_signer = &[&pool_signer_seeds[..]];

        if user_data.stake_amount - user_data.withdraw_amount > 0{
            let total_reward_amount = (user_data.stake_amount - user_data.withdraw_amount) * user_data.reward_rate / 10000 / 365 / 24 / 60 / 60 * (clock.unix_timestamp as u64 - user_data.last_time);

            let reward_cpi_accounts = Transfer {
                from: ctx.accounts.pool_token_account.to_account_info().clone(),
                to: ctx.accounts.user_token_account.to_account_info().clone(),
                authority: pool.to_account_info().clone(),
            };
    
            let reward_cpi_program = ctx.accounts.token_program.to_account_info().clone();
    
            let reward_cpi_ctx = CpiContext::new_with_signer(reward_cpi_program, reward_cpi_accounts, pool_signer);
    
            token::transfer(reward_cpi_ctx, total_reward_amount)?;

            pool.reward += total_reward_amount;
        }

        user_data.last_time = clock.unix_timestamp as u64;
        user_data.withdraw_amount = 0;
        user_data.withdrawable = false;
        user_data.request_time = 0;

        Ok(())
    }

    pub fn withdraw(
        ctx : Context<Withdraw>
        ) -> ProgramResult {
        msg!("+ withdraw");

        let user_data = &mut ctx.accounts.user_data;
        let pool = &mut ctx.accounts.pool;

        let clock = Clock::from_account_info(&ctx.accounts.clock)?;
        
        if user_data.withdraw_amount == 0{
            return Err(PoolError::InvalidWithdrawAmount.into());
        }

        if clock.unix_timestamp as u64 - user_data.request_time <= PERIOD as u64 * pool.withdraw_lock_period {
            return Err(PoolError::InvalidPeriod.into());
        }
   
        if user_data.stake_amount - user_data.withdraw_amount > 0{
            let total_reward_amount = (user_data.stake_amount - user_data.withdraw_amount) * user_data.reward_rate / 10000 / 365 / 24 / 60 / 60 * (clock.unix_timestamp as u64 - user_data.last_time);
            pool.reward += total_reward_amount;

            let reward_cpi_accounts = Transfer {
                from: ctx.accounts.pool_token_account.to_account_info().clone(),
                to: ctx.accounts.user_token_account.to_account_info().clone(),
                authority: pool.to_account_info().clone(),
            };

            let pool_signer_seeds = &[
                pool.rand.as_ref(),
                &[pool.bump],
            ];

            let pool_signer = &[&pool_signer_seeds[..]];
    
            let reward_cpi_program = ctx.accounts.token_program.to_account_info().clone();
    
            let reward_cpi_ctx = CpiContext::new_with_signer(reward_cpi_program, reward_cpi_accounts, pool_signer);
    
            token::transfer(reward_cpi_ctx, total_reward_amount)?;
        }
           
        let pool_signer_seeds = &[
            pool.rand.as_ref(),
            &[pool.bump],
        ];

        let pool_signer = &[&pool_signer_seeds[..]];

        if user_data.withdraw_amount == user_data.stake_amount {
            let cpi_accounts = Transfer {
                from: ctx.accounts.pool_nft_account.to_account_info().clone(),
                to: ctx.accounts.user_nft_account.to_account_info().clone(),
                authority: pool.to_account_info().clone(),
            };
    
            let cpi_program = ctx.accounts.token_program.to_account_info().clone();
    
            let token_cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, pool_signer);
    
            token::transfer(token_cpi_ctx, 1)?;
        }

        let token_cpi_accounts = Transfer {
            from: ctx.accounts.pool_token_account.to_account_info().clone(),
            to: ctx.accounts.user_token_account.to_account_info().clone(),
            authority: pool.to_account_info().clone(),
        };

        let token_cpi_program = ctx.accounts.token_program.to_account_info().clone();

        let token_cpi_ctx = CpiContext::new_with_signer(token_cpi_program, token_cpi_accounts, pool_signer);

        token::transfer(token_cpi_ctx, user_data.withdraw_amount)?;

        user_data.stake_amount -= user_data.withdraw_amount;
        user_data.withdraw_amount = 0;
        user_data.withdrawable = false;

        pool.tvl -= user_data.withdraw_amount;

        user_data.last_time = clock.unix_timestamp as u64;

        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(_bump : u8)]
pub struct InitPool<'info>{
    #[account(mut)]
    owner : Signer<'info>,

    #[account(init,
        seeds = [(*rand.key).as_ref()], 
        bump = _bump, 
        payer = owner, 
        space = 8 + POOL_SIZE)]
    pool : ProgramAccount<'info, Pool>,

    rand : AccountInfo<'info>,

    collection : AccountInfo<'info>,

    #[account(mut,
        constraint = fee_receiver.mint == token_mint.key())]
    fee_receiver : Account<'info, TokenAccount>,

    #[account(mut,
        constraint = trader.mint == token_mint.key())]
    trader : Account<'info, TokenAccount>,

    #[account(mut, 
        constraint = pool_ledger.owner == program_id)]
    pool_ledger : AccountInfo<'info>,

    #[account(owner = spl_token::id())]
    token_mint : Account<'info, Mint>,

    system_program : Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePool<'info> {
    #[account(mut)]
    owner : Signer<'info>,

    new_owner : AccountInfo<'info>,

    #[account(mut,
        has_one = owner,
        seeds = [pool.rand.as_ref()], 
        bump = pool.bump)]
    pool : ProgramAccount<'info, Pool>
}

#[derive(Accounts)]
pub struct ChangeLedger<'info> {
    #[account(mut)]
    owner : Signer<'info>,

    #[account(mut,
        has_one = owner,
        seeds = [pool.rand.as_ref()], 
        bump = pool.bump)]
    pool : ProgramAccount<'info, Pool>,

    #[account(mut, 
        constraint = pool_ledger.owner == program_id)]
    pool_ledger : AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(_bump : u8)]
pub struct InitUser<'info>{
    #[account(mut)]
    owner : Signer<'info>,

    #[account(mut,
        has_one = pool_ledger)]
    pool : ProgramAccount<'info, Pool>,

    #[account(mut)]
    pool_ledger : AccountInfo<'info>,

    #[account(init, 
        seeds = [(*owner.key).as_ref(), pool.key().as_ref()], 
        bump = _bump, 
        payer = owner, 
        space = 8+USER_DATA_SIZE)]
    user_data : ProgramAccount<'info, UserData>,

    #[account(owner = spl_token::id())]
    nft_mint : Account<'info, Mint>,

    #[account(mut,
        constraint = user_nft_account.owner == owner.key(),
        constraint = user_nft_account.mint == nft_mint.key())]
    user_nft_account : Account<'info, TokenAccount>,

    #[account(mut,
        constraint = pool_nft_account.owner == pool.key(),
        constraint = pool_nft_account.mint == nft_mint.key())]
    pool_nft_account : Account<'info, TokenAccount>,

    token_program:Program<'info, Token>,

    system_program : Program<'info, System>,    
}

#[derive(Accounts)]
pub struct UpdateUser<'info>{
    #[account(mut)]
    owner : Signer<'info>,

    #[account(mut,
        has_one = pool_ledger)]
    pool : ProgramAccount<'info, Pool>,

    #[account(mut)]
    pool_ledger : AccountInfo<'info>,

    #[account(mut, 
        seeds = [(*owner.key).as_ref(), user_data.pool.key().as_ref()], 
        bump = user_data.bump)]
    user_data : ProgramAccount<'info, UserData>,

    #[account(owner = spl_token::id())]
    nft_mint : Account<'info, Mint>,

    #[account(mut,
        constraint = user_nft_account.owner == owner.key(),
        constraint = user_nft_account.mint == nft_mint.key())]
    user_nft_account : Account<'info, TokenAccount>,

    #[account(mut,
        constraint = pool_nft_account.owner == pool.key(),
        constraint = pool_nft_account.mint == nft_mint.key())]
    pool_nft_account : Account<'info, TokenAccount>,

    token_program:Program<'info, Token>,

    system_program : Program<'info, System>,    
}

#[derive(Accounts)]
pub struct SetRate<'info>{
    #[account(mut)]
    owner : Signer<'info>,

    #[account(mut)]
    pool_ledger : AccountInfo<'info>
}

#[derive(Accounts)]
pub struct SetList<'info>{
    #[account(mut)]
    owner : Signer<'info>,

    #[account(mut)]
    pool_ledger : AccountInfo<'info>
}

#[derive(Accounts)]
pub struct Deposit<'info>{
    #[account(mut)]
    owner : Signer<'info>,

    #[account(mut)]
    pool : ProgramAccount<'info,Pool>,

    #[account(mut)]
    pool_ledger : AccountInfo<'info>,        

    #[account(mut, 
        seeds = [(*owner.key).as_ref(), pool.key().as_ref()],
        bump = user_data.bump)]
    user_data : ProgramAccount<'info, UserData>,

    #[account(mut,
        constraint = user_token_account.owner == owner.key(),
        constraint = user_token_account.mint == pool.token_mint)]
    user_token_account:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = pool_token_account.owner == pool.key(),
        constraint = pool_token_account.mint == pool.token_mint)]
    pool_token_account:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = trader.key() == pool.trader,
        constraint = trader.mint == pool.token_mint)]
    trader:Account<'info, TokenAccount>,

    token_program:Program<'info, Token>,

    clock : AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct InstantWithdraw<'info>{
    #[account(mut)]
    owner : Signer<'info>,

    #[account(mut,
        has_one = pool_ledger)]
    pool : ProgramAccount<'info,Pool>,

    #[account(mut)]
    pool_ledger : AccountInfo<'info>,        

    #[account(mut, 
        seeds = [(*owner.key).as_ref(), pool.key().as_ref()], 
        bump = user_data.bump)]
    user_data : ProgramAccount<'info, UserData>,

    #[account(mut,
        constraint = user_nft_account.owner == owner.key(),
        constraint = user_nft_account.mint == user_data.nft_mint)]
    user_nft_account : Account<'info, TokenAccount>,

    #[account(mut,
        constraint = pool_nft_account.owner == pool.key(),
        constraint = pool_nft_account.mint == user_data.nft_mint)]
    pool_nft_account : Account<'info, TokenAccount>,

    #[account(mut,
        constraint = user_token_account.owner == owner.key(),
        constraint = user_token_account.mint == pool.token_mint,
        owner = spl_token::id())]
    user_token_account:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = pool_token_account.owner == pool.key(),
        constraint = pool_token_account.mint == pool.token_mint,
        owner = spl_token::id())]
    pool_token_account:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = fee_receiver.key() == pool.fee_receiver,
        constraint = fee_receiver.mint == pool.token_mint)]
    fee_receiver:Account<'info, TokenAccount>,

    token_program:Program<'info, Token>,

    clock : AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct EmergencyWithdraw<'info>{
    #[account(mut)]
    owner : Signer<'info>,

    #[account(mut)]
    pool : ProgramAccount<'info,Pool>,

    #[account(mut,
        constraint = user_token_account.owner == owner.key(),
        constraint = user_token_account.mint == pool.token_mint,
        owner = spl_token::id())]
    user_token_account:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = pool_token_account.owner == pool.key(),
        constraint = pool_token_account.mint == pool.token_mint,
        owner = spl_token::id())]
    pool_token_account:Account<'info, TokenAccount>,

    token_program:Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RequestWithdraw<'info>{
    #[account(mut)]
    owner : Signer<'info>,

    #[account(mut)]
    pool : ProgramAccount<'info, Pool>,

    #[account(mut, 
        seeds = [(*owner.key).as_ref(), user_data.pool.key().as_ref()], 
        bump = user_data.bump)]
    user_data : ProgramAccount<'info, UserData>,

    #[account(mut,
        constraint = user_token_account.owner == owner.key(),
        constraint = user_token_account.mint == pool.token_mint,
        owner = spl_token::id())]
    user_token_account:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = pool_token_account.owner == pool.key(),
        constraint = pool_token_account.mint == pool.token_mint,
        owner = spl_token::id())]
    pool_token_account:Account<'info, TokenAccount>,

    token_program:Program<'info, Token>,

    clock : AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CancelRequest<'info>{
    #[account(mut)]
    owner : Signer<'info>,

    #[account(mut)]
    pool : ProgramAccount<'info, Pool>,

    #[account(mut, 
        seeds = [(*owner.key).as_ref(), user_data.pool.key().as_ref()], 
        bump = user_data.bump)]
    user_data : ProgramAccount<'info, UserData>,

    #[account(mut,
        constraint = user_token_account.owner == owner.key(),
        constraint = user_token_account.mint == pool.token_mint,
        owner = spl_token::id())]
    user_token_account:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = pool_token_account.owner == pool.key(),
        constraint = pool_token_account.mint == pool.token_mint,
        owner = spl_token::id())]
    pool_token_account:Account<'info, TokenAccount>,

    token_program:Program<'info, Token>,

    clock : AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info>{
    #[account(mut)]
    owner : Signer<'info>,

    #[account(mut,
        has_one = pool_ledger)]
    pool : ProgramAccount<'info,Pool>,

    #[account(mut)]
    pool_ledger : AccountInfo<'info>,        

    #[account(mut, 
        seeds = [(*owner.key).as_ref(), pool.key().as_ref()], 
        bump = user_data.bump)]
    user_data : ProgramAccount<'info, UserData>,

    #[account(mut,
        constraint = user_nft_account.owner == owner.key(),
        constraint = user_nft_account.mint == user_data.nft_mint)]
    user_nft_account : Account<'info, TokenAccount>,

    #[account(mut,
        constraint = pool_nft_account.owner == pool.key(),
        constraint = pool_nft_account.mint == user_data.nft_mint)]
    pool_nft_account : Account<'info, TokenAccount>,

    #[account(mut,
        constraint = user_token_account.owner == owner.key(),
        constraint = user_token_account.mint == pool.token_mint,
        owner = spl_token::id())]
    user_token_account:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = pool_token_account.owner == pool.key(),
        constraint = pool_token_account.mint == pool.token_mint,
        owner = spl_token::id())]
    pool_token_account:Account<'info, TokenAccount>,

    token_program:Program<'info, Token>,

    clock : AccountInfo<'info>,
}

pub const PERIOD : usize = 24 * 60 * 60;

pub const POOL_SIZE : usize = 32 + 32 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 1;
#[account]
pub struct Pool{
    pub owner : Pubkey,
    pub token_mint : Pubkey,
    pub collection : Pubkey,
    pub pool_ledger : Pubkey,
    pub rand : Pubkey,
    pub fee_receiver : Pubkey,
    pub trader : Pubkey,
    pub start_at : u64,
    pub deposit_limit : u64,
    pub reward_lock_period : u64,
    pub withdraw_lock_period : u64,
    pub fee : u64,
    pub tvl : u64,
    pub reward : u64,
    pub decimals : u8,
    pub bump : u8,
}

pub const USER_DATA_SIZE : usize = 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1 + 1;
#[account]
pub struct UserData{
    pub owner : Pubkey,
    pub pool : Pubkey,
    pub nft_mint : Pubkey,
    pub stake_amount : u64,
    pub reward_rate : u64,
    pub last_time : u64,
    pub withdraw_amount : u64,
    pub request_time : u64,
    pub withdrawable : bool,
    pub bump : u8,
}

pub const MAX_LEN : usize = 1111;
pub const POOL_LEDGER_SIZE : usize = 32 + 1 + 4 + 4 + DAILY_LEDGER_SIZE * MAX_LEN;
#[account]
#[derive(Default)]
pub struct RateList{
    pub pool : Pubkey,
    pub rasing_rate : u8,
    pub last_number : u32,
    pub ledger : Vec<RewardRate>
}

pub const DAILY_LEDGER_SIZE : usize = 8 + 32;
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct RewardRate{
    pub reward_rate : u64,
    pub mint : Pubkey,
}

pub fn set_reward_rate(
    a: &mut AccountInfo,
    index : usize,
    daily_ledger : RewardRate,
    ){
    let mut arr = a.data.borrow_mut();
    let data_array = daily_ledger.try_to_vec().unwrap();
    let vec_start = 8 + 32 + 1 + 4 + 4 + DAILY_LEDGER_SIZE * index;
    for i in 0..data_array.len(){
        arr[vec_start+i] = data_array[i];
    }
}

pub fn set_rasing_rate(
    a: &mut AccountInfo,
    rasing_rate : u8,
    ){
    let mut arr = a.data.borrow_mut();
    let data_array = rasing_rate.try_to_vec().unwrap();
    let vec_start = 40;
    for i in 0..data_array.len() {
        arr[vec_start+i] = data_array[i];
    }    
}

pub fn set_last_number(
    a: &mut AccountInfo,
    number : u32,
    ){
    let mut arr = a.data.borrow_mut();
    let data_array = number.try_to_vec().unwrap();
    let vec_start = 41;
    for i in 0..data_array.len() {
        arr[vec_start+i] = data_array[i];
    }    
}

pub fn get_reward_rate(
    a : &AccountInfo,
    index : usize,
    ) -> core::result::Result<RewardRate, ProgramError> {
    let arr = a.data.borrow();
    let vec_start = 8 + 32 + 1 + 4 + 4 + DAILY_LEDGER_SIZE * index;
    let data_array = &arr[vec_start..vec_start+DAILY_LEDGER_SIZE];
    let daily_ledger : RewardRate = RewardRate::try_from_slice(data_array)?;
    Ok(daily_ledger)
}

pub fn get_rasing_rate(
    a : &AccountInfo
    ) -> core::result::Result<u8, ProgramError>{
    let arr= a.data.borrow();
    let data_array = &arr[40..41];
    let rasing_rate : u8 = u8::try_from_slice(data_array)?;
    Ok(rasing_rate)
}

pub fn get_last_number(
    a : &AccountInfo
    ) -> core::result::Result<u32, ProgramError>{
    let arr= a.data.borrow();
    let data_array = &arr[41..45];
    let last_number : u32 = u32::try_from_slice(data_array)?;
    Ok(last_number)
}

#[error]
pub enum PoolError {
    #[msg("Token mint to failed")]
    TokenMintToFailed,

    #[msg("Token set authority failed")]
    TokenSetAuthorityFailed,

    #[msg("Token transfer failed")]
    TokenTransferFailed,

    #[msg("Token burn failed")]
    TokenBurnFailed,

    #[msg("Invalid Ranking")]
    InvalidRanking,

    #[msg("Invalid time")]
    InvalidTime,

    #[msg("Invalid pool ledger")]
    InvalidPoolLedger,

    #[msg("Invalid period")]
    InvalidPeriod,

    #[msg("Invalid metadata extended account")]
    InvalidMetadataExtended,

    #[msg("Invalid token mint")]
    InvalidTokenMint,

    #[msg("Invalid stake amount")]
    InvalidStakeAmount,

    #[msg("Invalid withdraw amount")]
    InvalidWithdrawAmount,
}