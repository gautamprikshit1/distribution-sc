#![no_std]

elrond_wasm::imports!();

mod pause;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[elrond_wasm::contract]
pub trait Distribution: pause::PauseModule {
    #[init]
    fn init(&self, dist_token_id: EgldOrEsdtTokenIdentifier, dist_token_price: BigUint) {
        self.distributable_token_id().set_if_empty(&dist_token_id);
        self.distributable_token_price()
            .set_if_empty(&dist_token_price);
    }

    #[only_owner]
    #[endpoint(updatePrice)]
    fn update_price(&self, dist_token_price: BigUint) -> SCResult<()> {
        self.distributable_token_price().set(&dist_token_price);
        Ok(())
    }

    #[only_owner]
    #[endpoint(updateBuyLimit)]
    fn update_buy_limit(&self, limit_amount: BigUint) -> SCResult<()> {
        self.buy_limit().set(&limit_amount);
        Ok(())
    }

    #[only_owner]
    #[payable("*")]
    #[endpoint]
    fn deposit(&self, #[payment_token] token: EgldOrEsdtTokenIdentifier) -> SCResult<()> {
        require!(
            token == self.distributable_token_id().get(),
            "Invalid token!"
        );
        Ok(())
    }

    #[only_owner]
    #[endpoint]
    fn claim(&self) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        let balance = self
            .blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0);
        require!(balance > 0, "No funds to claim!");
        self.send()
            .direct(&caller, &EgldOrEsdtTokenIdentifier::egld(), 0, &balance);
        Ok(())
    }

    #[payable("EGLD")]
    #[endpoint]
    fn buy(&self, #[payment_amount] paid_amount: BigUint) -> SCResult<()> {
        require!(paid_amount != 0, "zero, really??");
        require!(self.not_paused(), "Sale has been paused");

        if !self.buy_limit().is_empty() {
            require!(paid_amount <= self.buy_limit().get(), "Buy limit exceeded");
        }
        let caller = self.blockchain().get_caller();
        let dist_token_id = self.distributable_token_id().get();
        let price_per_token = self.distributable_token_price().get();
        let available_token_amount = self.blockchain().get_sc_balance(&dist_token_id, 0);

        let token_amount = &paid_amount / &price_per_token;
        require!(
            token_amount <= available_token_amount,
            "Not enough tokens available."
        );

        self.send()
            .direct(&caller, &dist_token_id, 0, &token_amount);
        Ok(())
    }

    #[payable("*")]
    #[endpoint]
    fn burn(
        &self,
        #[payment_token] payment_token: EgldOrEsdtTokenIdentifier,
        #[payment_amount] payment_amount: BigUint,
    ) -> SCResult<()> {
        require!(
            payment_token == self.distributable_token_id().get(),
            "invalid token"
        );

        self.send()
            .esdt_local_burn(&payment_token.unwrap_esdt(), 0, &payment_amount);
        self.burned_tokens()
            .update(|current| *current += payment_amount);

        Ok(())
    }

    #[view(getDistributableTokenId)]
    #[storage_mapper("distributableToken")]
    fn distributable_token_id(&self) -> SingleValueMapper<EgldOrEsdtTokenIdentifier>;

    #[view(getDistributablePrice)]
    #[storage_mapper("distributablePrice")]
    fn distributable_token_price(&self) -> SingleValueMapper<BigUint>;

    #[view(getBuyLimit)]
    #[storage_mapper("buyLimit")]
    fn buy_limit(&self) -> SingleValueMapper<BigUint>;

    #[view(getBurnedAmount)]
    #[storage_mapper("burnedAmount")]
    fn burned_tokens(&self) -> SingleValueMapper<BigUint>;
}
