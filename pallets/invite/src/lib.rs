#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::Saturating;
use frame_support::{
	decl_module, decl_event, decl_error, ensure,
	dispatch::DispatchError,
	traits::{Currency, StoredMap},
	weights::Pays,
};
use frame_system::ensure_root;
use codec::{Encode, Decode};

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Currency: Currency<Self::AccountId>;
}

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
		UserCreated(Vec<u8>, AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// Could not generate a valid AccountId
		CouldNotDecode,
		/// Account already exists.
		AlreadyInvited,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		#[weight = (0, Pays::No)]
		fn create_account(origin, seed: Vec<u8>) {
			ensure_root(origin)?;
			let account = Self::get_account(&seed)?;
			// Check account doesn't already exist.
			ensure!(!frame_system::Module::<T>::is_explicit(&account), Error::<T>::AlreadyInvited);
			// Give them some balance.
			let multiplier: BalanceOf<T> = 1_000_000u32.into();
			let balance = T::Currency::minimum_balance().saturating_mul(multiplier);
			T::Currency::make_free_balance_be(&account, balance);
			// Increment their nonce so that their account cannot be deleted.
			frame_system::Module::<T>::inc_ref(&account);
			Self::deposit_event(RawEvent::UserCreated(seed, account));
		}
	}
}

impl<T: Config> Module<T> {
	pub fn get_account(seed: &[u8]) -> Result<T::AccountId, DispatchError> {
		let entropy = (b"syn/invite", seed).using_encoded(blake2_256);
		T::AccountId::decode(&mut &entropy[..]).map_err(|_| Error::<T>::CouldNotDecode.into())
	}
}
