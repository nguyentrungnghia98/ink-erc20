#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {
	use ink_prelude::{
        string::String,
    };
	use ink_storage::{traits::SpreadAllocate, Mapping};

	#[ink(event)]
	pub struct Transferred {
		from: Option<AccountId>,
		to: Option<AccountId>,
		value: Balance,
	}

	#[ink(storage)]
	#[derive(SpreadAllocate)]
    pub struct Erc20 {
		_owner: AccountId,
        _balances: Mapping<AccountId, Balance>,
		_total_supply: Balance,
		_name: String,
		_symbol: String,
    }

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(total_supply: Balance, name: String, symbol: String) -> Self {
			let owner = Self::env().caller();

			ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract._balances.insert(&owner, &total_supply);
				contract._owner = owner.clone();
				contract._total_supply = total_supply;
				contract._name = name;
				contract._symbol = symbol;

				Self::env().emit_event(Transferred {
					from: None,
					to: Some(owner.clone()),
					value: total_supply,
				});
            })

        }

        #[ink(message)]
        pub fn name(&self) -> String {
            self._name.clone()
        }

        #[ink(message)]
        pub fn symbol(&self) -> String {
            self._symbol.clone()
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self._total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> Balance {
            self._balances.get(account).unwrap_or(0)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: Balance) -> bool {
			let owner = self.env().caller();
			self._transfer(&owner, &to, amount);

			true
        }

        #[ink(message)]
        pub fn mint(&mut self, amount: Balance) -> bool {
			let owner = self.env().caller();
			self.only_allowed_caller();
			self._mint(&owner, amount);

			true
        }

        #[ink(message)]
        pub fn burn(&mut self, amount: Balance) -> bool {
			let owner = self.env().caller();
			self.only_allowed_caller();
			self._burn(&owner, amount);

			true
        }

		fn _transfer(&mut self, from: &AccountId, to: &AccountId, amount: Balance) {
			let from_balance = self._balances.get(from).unwrap_or(0);
			let to_balance = self._balances.get(to).unwrap_or(0);
			assert!(from_balance >= amount, "ERC20: transfer amount exceeds balance");

			let new_from_balance: Balance = from_balance - amount;
			let new_to_balance: Balance = to_balance + amount;
			self._balances.insert(from, &new_from_balance);
			self._balances.insert(to, &new_to_balance);

			Self::env().emit_event(Transferred {
				from: Some(from.clone()),
				to: Some(to.clone()),
				value: amount,
			});
		}

		fn _mint(&mut self, account: &AccountId, amount: Balance) {
			self._total_supply += amount;

			let account_balance = self._balances.get(account).unwrap_or(0);
			self._balances.insert(account, &(account_balance + amount));

			Self::env().emit_event(Transferred {
				from: None,
				to: Some(account.clone()),
				value: amount,
			});
		}

		fn _burn(&mut self, account: &AccountId, amount: Balance) {
			let balance = self._balances.get(account).unwrap_or(0);
			assert!(balance >= amount, "ERC20: burn amount exceeds balance");
			self._total_supply -= amount;
			self._balances.insert(account, &(balance - amount));

			Self::env().emit_event(Transferred {
				from: Some(account.clone()),
				to: None,
				value: amount,
			});
		}

		fn only_allowed_caller(&self) {
            assert!(
                self._owner == self.env().caller(),
                "only_allowed_caller: this caller is not allowed",
            );
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_constructor_works() {
            let erc20 = Erc20::new(1000, "Polkadot".to_string(), "DOT".to_string());
            assert_eq!(erc20.name(), "Polkadot");
            assert_eq!(erc20.symbol(), "DOT");
            assert_eq!(erc20.total_supply(), 1000);
        }

		#[ink::test]
        fn it_transfer_works() {
    		let mut erc20 = Erc20::new(1000, "Polkadot".to_string(), "DOT".to_string());
			let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

			let bob_balance = erc20.balance_of(accounts.bob);

			assert_eq!(bob_balance, 0);

            assert_eq!(erc20.transfer(accounts.bob, 1), true);

            assert_eq!(erc20.balance_of(accounts.bob), 1);

            // ink_env::debug_println!("balance 1 {:?} - {:?}", bob_balance, erc20.balance_of(accounts.bob));
			let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 2);
		}

		#[ink::test]
        fn it_mint_works() {
    		let mut erc20 = Erc20::new(1000, "Polkadot".to_string(), "DOT".to_string());
			let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

			let alice_balance = erc20.balance_of(accounts.alice);

            assert_eq!(erc20.mint(10), true);

            assert_eq!(erc20.balance_of(accounts.alice), alice_balance + 10);

			let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 2);
		}

		#[ink::test]
        fn it_burn_works() {
    		let mut erc20 = Erc20::new(1000, "Polkadot".to_string(), "DOT".to_string());
			let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>();

			let alice_balance = erc20.balance_of(accounts.alice);

            assert_eq!(erc20.burn(10), true);

            assert_eq!(erc20.balance_of(accounts.alice), alice_balance - 10);

			let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 2);
		}
    }
}
