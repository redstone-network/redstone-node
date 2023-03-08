//! # Defense Pallet
//!
//! A secure module for protecting users' asset
//!
//! ## Overview
//!
//! The Defense module provides functionality for asset protection of asset
//!
//! Including:
//!
//! * set_transfer_limitation
//! * set_freeze_configuration
//! * safe_transfer
//! * freeze_account

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	traits::WrapperKeepOpaque,
	weights::{GetDispatchInfo, PostDispatchInfo},
};

use scale_info::TypeInfo;
use sp_runtime::{
	offchain::{http, Duration},
	traits::Dispatchable,
	transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
	RuntimeDebug,
};
use sp_std::cmp::{Eq, PartialEq};

use frame_system::offchain::{
	AppCrypto, CreateSignedTransaction, SignedPayload, Signer, SigningTypes,
};
use sp_core::crypto::KeyTypeId;

pub type OpaqueCall<T> = WrapperKeepOpaque<<T as Config>::Call>;

pub type CallHash = [u8; 32];

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocwd");
pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};
	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;
	// implemented for ocw-runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

/// an enum, each instance represents a different freeze configuration when malicious transfers
/// occur: freeze the account temporarily or permanently

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ActionConfig {
	TimeFreeze(u64),     // freeze account temporary
	AccountFreeze(bool), // freeze account permanent
}

use serde::{Deserialize, Serialize};
use serde_json::Result as R;
#[derive(Debug, Deserialize)]
struct TracingResult {
	value: u64,
}

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use data_encoding::BASE64;
	use frame_support::{
		fail,
		inherent::Vec,
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, ReservableCurrency},
	};
	use frame_system::{offchain::SendUnsignedTransaction, pallet_prelude::*};

	use sp_runtime::traits::{One, SaturatedConversion};

	pub use primitives::tx_tracing::AccountStatusInfo;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct TrancedAccountPayload<Public, AccountId> {
		pub account: AccountId,
		pub public: Public,
	}

	impl<T: SigningTypes> SignedPayload<T> for TrancedAccountPayload<T::Public, T::AccountId> {
		fn public(&self) -> T::Public {
			self.public.clone()
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;

		/// The overarching call type.
		type Call: Parameter
			+ Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
			+ GetDispatchInfo
			+ From<frame_system::Call<Self>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn next_tracing_account_index)]
	pub type NextTracingAccountIndex<T: Config> = StorageValue<_, u64>;

	#[pallet::storage]
	#[pallet::getter(fn analyze_account_map)]
	pub(super) type MapTracingAccount<T: Config> = StorageMap<_, Twox64Concat, u64, T::AccountId>;

	/// store the accounts analyzed status by AI
	#[pallet::storage]
	#[pallet::getter(fn tracing_address)]
	pub(super) type TracingAccount<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	/// store the accounts analyzed status by AI
	#[pallet::storage]
	#[pallet::getter(fn abnormal_address)]
	pub(super) type AbnormalAccount<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	/// event emitted when the user performs an action successfully
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TrancingAccountSet(T::AccountId),
	}

	/// events which indicate that users' call execute fail
	#[pallet::error]
	pub enum Error<T> {
		TracingAccountHasBeenSet,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// set address who need to be traced
		#[pallet::weight(10_000)]
		pub fn set_tracing_account(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match TracingAccount::<T>::get(who.clone()) {
				Some(_) => {
					ensure!(
						!TracingAccount::<T>::contains_key(&who),
						Error::<T>::TracingAccountHasBeenSet
					);
				},
				None => {
					TracingAccount::<T>::mutate(who.clone(), |v| *v = Some(true));
					let cur_tracing_account_index =
						NextTracingAccountIndex::<T>::get().unwrap_or_default();

					MapTracingAccount::<T>::insert(cur_tracing_account_index, who.clone());

					NextTracingAccountIndex::<T>::put(
						cur_tracing_account_index.saturating_add(One::one()),
					);

					Self::deposit_event(Event::TrancingAccountSet(who.clone()));
				},
			}

			Ok(())
		}

		// set abnormal address

		#[pallet::weight(0)]
		pub fn set_account_as_abnormal(
			origin: OriginFor<T>,
			traced_account_payload: TrancedAccountPayload<T::Public, T::AccountId>,
			_signature: T::Signature,
		) -> DispatchResultWithPostInfo {
			let _who = ensure_none(origin)?;

			match AbnormalAccount::<T>::get(traced_account_payload.account.clone()) {
				Some(_) => {},
				None => {
					AbnormalAccount::<T>::mutate(traced_account_payload.account.clone(), |v| {
						*v = Some(true)
					});
					log::info!(
						"-------------------------------- find an abnormal account and marked "
					);
				},
			}

			Ok(().into())
		}
	}

	/// all notification will be send via offchain_worker, it is more efficient
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("Hi World from tx-tracing-pallet workers!: {:?}", block_number);

			let cur_tracing_account_index = NextTracingAccountIndex::<T>::get().unwrap_or_default();

			for i in 0..cur_tracing_account_index {
				match MapTracingAccount::<T>::get(i) {
					Some(account) => match AbnormalAccount::<T>::get(account.clone()) {
						Some(_) => {},
						None => {
							let res = Self::send_tracing_account_to_ai(account.clone());
							let _res = Self::set_abnormal_account(account.clone(), res);
						},
					},
					_ => {},
				}
			}
		}
	}

	/// helper functions
	impl<T: Config> Pallet<T> {
		fn send_tracing_account_to_ai(who: T::AccountId) -> Result<u64, http::Error> {
			let addr_vec = who.clone().encode();

			let address = BASE64.encode(&addr_vec[..]);

			let basic_url = "http://127.0.0.1:3030/address?";

			let url = &scale_info::prelude::format!("{}addr={}", basic_url, address)[..];
			// let url = &scale_info::prelude::format!("{}", basic_url);

			let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(10_000));

			let request = http::Request::get(url);

			let pending = request.deadline(deadline).send().map_err(|e| {
				log::info!("---------get pending error: {:?}", e);
				http::Error::IoError
			})?;

			let response =
				pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
			if response.code != 200 {
				log::warn!("Unexpected status code: {}", response.code);
				return Err(http::Error::Unknown)
			} else {
				log::info!("get analyze account status successfully")
			}
			let body = response.body().collect::<Vec<u8>>();
			let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
				log::warn!("No UTF8 body");
				http::Error::Unknown
			})?;

			let message: Result<TracingResult, _> = serde_json::from_str(body_str);

			match message {
				Ok(result) => {
					log::info!("get analyze result {:?}", result);
					if result.value == 1 {
						return Ok(1)
					}
				},

				Err(err) => {
					log::info!("convert body to json failed: {}", err)
				},
			};

			Ok(0)
		}

		fn set_abnormal_account(
			who: T::AccountId,
			res: Result<u64, http::Error>,
		) -> Result<u64, Error<T>> {
			if let Ok(is_abnormal) = res {
				if is_abnormal == 1 {
					if let Some((_, res)) = Signer::<T, T::AuthorityId>::any_account()
						.send_unsigned_transaction(
							// this line is to prepare and return payload
							|account| TrancedAccountPayload {
								account: who.clone(),
								public: account.public.clone(),
							},
							|payload, signature| Call::set_account_as_abnormal {
								traced_account_payload: payload,
								signature,
							},
						) {
						match res {
							Ok(()) => {
								log::info!(
									"-----unsigned tx with signed payload successfully sent."
								);
							},
							Err(()) => {
								log::error!("---sending unsigned tx with signed payload failed.");
							},
						};
					} else {
						// The case of `None`: no account is available for sending
						log::error!("----No local account available");
					}
				}
			}

			Ok(0)
		}
	}

	/// configure unsigned tx, use it to update onchain status of notification, so that
	/// notifications will not send repeatedly
	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			let valid_tx = |provide| {
				ValidTransaction::with_tag_prefix("ocw-tx-tracing")
					.priority(T::UnsignedPriority::get())
					.and_provides([&provide])
					.longevity(3)
					.propagate(true)
					.build()
			};

			match call {
				Call::set_account_as_abnormal {
					traced_account_payload: ref payload,
					ref signature,
				} => {
					let signature_valid =
						SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
					if !signature_valid {
						return InvalidTransaction::BadProof.into()
					}

					valid_tx(b"set_account_as_abnormal".to_vec())
				},
				_ => InvalidTransaction::Call.into(),
			}
		}
	}

	impl<T: Config> AccountStatusInfo<T::AccountId> for Pallet<T> {
		fn get_account_status(account: T::AccountId) -> Option<bool> {
			AbnormalAccount::<T>::get(account)
		}
	}
}
