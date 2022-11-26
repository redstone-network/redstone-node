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
//! * set_transfer_limit
//! * set_risk_management
//! * safe_transfer

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

use pallet_difttt::Action;

use scale_info::TypeInfo;
use sp_runtime::{
	offchain::{http, Duration},
	traits::Dispatchable,
	RuntimeDebug,
};
use sp_std::cmp::{Eq, PartialEq};

use frame_system::offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer};
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

	pub struct OcwAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OcwAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for OcwAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

/// a enum to organize different limitations of transferring money
/// there are two kind of predefined limitations
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum TransferLimit<Balance> {
	AmountLimit(Balance), // amount limit per transaction
	FrequencyLimit(u64, u64), /* the first parameter is times limit, the second parameter is
	                       * blocks limit */
}

/// a enum to organize different protect actions  when illegal transactions occur
/// there are two kind of predefined actions
#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RiskManagement {
	TimeFreeze(u64),     // freeze account for a period of time
	AccountFreeze(bool), // freeze account forever
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		fail,
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{One, SaturatedConversion};
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	use codec::alloc::string::ToString;
	use data_encoding::BASE64;
	use frame_support::inherent::Vec;

	use primitives::{
		custom_call::CustomCallInterface, permission_capture::PermissionCaptureInterface,
	};
	use sp_io::hashing::blake2_256;

	use pallet_notification::NotificationInfoInterface;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		type PermissionCaptureInterface: PermissionCaptureInterface<
			Self::AccountId,
			OpaqueCall<Self>,
			BalanceOf<Self>,
		>;

		/// The overarching call type.
		type Call: Parameter
			+ Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
			+ GetDispatchInfo
			+ From<frame_system::Call<Self>>;

		type CustomCallInterface: CustomCallInterface<Self::AccountId, BalanceOf<Self>>;
		type Notification: NotificationInfoInterface<Self::AccountId, Action<Self::AccountId>>;
	}

	/// store transfer limit
	#[pallet::storage]
	#[pallet::getter(fn transfer_frequency_limit_map)]
	pub(super) type MapFrequencyLimit<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (T::BlockNumber, TransferLimit<BalanceOf<T>>)>;

	/// store transfer limit
	#[pallet::storage]
	#[pallet::getter(fn freeze_account_temporary_map)]
	pub(super) type MapFreezeAccountTemporary<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (T::BlockNumber, RiskManagement)>;

	/// store default freeze config
	#[pallet::storage]
	#[pallet::getter(fn default_freeze_account_temporary_map)]
	pub(super) type DefaultFreezeAccountTemporary<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (T::BlockNumber, u64)>;

	/// storage owner,id of transfer limit instance
	#[pallet::storage]
	#[pallet::getter(fn transfer_limit_owner)]
	pub type TransferLimitOwner<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		u64,
		TransferLimit<BalanceOf<T>>,
		OptionQuery,
	>;

	/// store owner,id of risk management instance
	#[pallet::storage]
	#[pallet::getter(fn risk_management_owner)]
	pub type RiskManagementOwner<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		u64,
		RiskManagement,
		OptionQuery,
	>;

	/// store account block status
	#[pallet::storage]
	#[pallet::getter(fn block_account)]
	pub type BlockAccount<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	/// store account block time
	#[pallet::storage]
	#[pallet::getter(fn block_time)]
	pub type BlockTime<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	/// store account default freeze
	#[pallet::storage]
	#[pallet::getter(fn default_freeze)]
	pub type DefaultFreeze<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	#[pallet::storage]
	#[pallet::getter(fn next_account_index)]
	pub type NextNotifyAccountIndex<T: Config> = StorageValue<_, u64>;

	/// store all accounts need to notify
	#[pallet::storage]
	#[pallet::getter(fn notify_account_map)]
	pub(super) type MapNotifyAccount<T: Config> = StorageMap<_, Twox64Concat, u64, T::AccountId>;

	/// store account's method of notification as email
	#[pallet::storage]
	#[pallet::getter(fn mail_status)]
	pub(super) type MailStatus<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	/// store account's method of notification as slack
	#[pallet::storage]
	#[pallet::getter(fn slack_status)]
	pub(super) type SlackStatus<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	/// store account's method of notification as discord
	#[pallet::storage]
	#[pallet::getter(fn discord_status)]
	pub(super) type DiscordStatus<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	/// events which indicate that users' call execute successfully
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TransferAmountLimitSet(T::AccountId, TransferLimit<BalanceOf<T>>), // set amount limit
		TransferFrequencyLimitSet(T::AccountId, TransferLimit<BalanceOf<T>>), // set times limit
		TransferAmountLimitUpdated(T::AccountId, TransferLimit<BalanceOf<T>>), // update amount limit
		TransferFrequencyLimitUpdated(T::AccountId, TransferLimit<BalanceOf<T>>), // update times limit
		RiskManagementTimeFreezeSet(T::AccountId, RiskManagement),         /* freeze amount for
		                                                                    * a period of time */
		RiskManagementAccountFreezeSet(T::AccountId, RiskManagement), // freeze account forever
		FreezeAccountForever(T::AccountId),
		FreezeAccountTemporary(T::AccountId),
		TransferSuccess(T::AccountId, T::AccountId, BalanceOf<T>), // transfer success
		FreezeAccountSuccess(T::AccountId),
	}

	/// events which indicate that users' call execute fail
	#[pallet::error]
	pub enum Error<T> {
		TransferAmountLimitHasSet,
		TransferFrequencyLimitHasSet,
		FreezeTimeHasSet,    // set account freeze time duplicate
		FreezeAccountHasSet, // set freeze account duplicate
		TransferValueTooLarge, /* transfer amount reach out transfer amount
		                      * limitation */
		TransferTimesTooMany, // transfer times reach out transfer times limitation
		AccountHasBeenFrozenForever, // transfer again when account has been frozen forever
		AccountHasBeenFrozenTemporary, // transfer again when account has been frozen temporarily
		PermissionTakenAccountHasPaddingCall, // frozen account has been in pending list
		PermissionTakenAccountCallMustBeApproved, // take frozen account must be approved
	}

	/// a function can set different transfer limitations, such as amount per transaction, times per
	/// 100 blocks. All these limitations will be triggered when someone try to stole money from
	/// current account at the same time, transaction will fail and emit error
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn set_transfer_limit(
			origin: OriginFor<T>,
			transfer_limit: TransferLimit<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match transfer_limit {
				TransferLimit::AmountLimit(_amount) => {
					if TransferLimitOwner::<T>::contains_key(&who, 1) == true {
						ensure!(
							!TransferLimitOwner::<T>::contains_key(&who, 1),
							Error::<T>::TransferAmountLimitHasSet
						);
					} else {
						TransferLimitOwner::<T>::insert(&who, 1, transfer_limit.clone());
						Self::deposit_event(Event::TransferAmountLimitSet(
							who.clone(),
							transfer_limit,
						));
						log::info!("--------------------------------set transfer amount limit");
					}
				},
				TransferLimit::FrequencyLimit(_max_available_amount, _blocks_limit) => {
					if TransferLimitOwner::<T>::contains_key(&who, 2) == true {
						ensure!(
							!TransferLimitOwner::<T>::contains_key(&who, 2),
							Error::<T>::TransferFrequencyLimitHasSet
						)
					} else {
						TransferLimitOwner::<T>::insert(&who, 2, transfer_limit.clone());
						MapFrequencyLimit::<T>::insert(
							&who,
							(frame_system::Pallet::<T>::block_number(), transfer_limit.clone()),
						);
						Self::deposit_event(Event::TransferFrequencyLimitSet(
							who.clone(),
							transfer_limit,
						));

						log::info!("--------------------------------set transfer times limit");
					}
				},
			}
			Ok(())
		}

		/// a function to set some protections when illegal transactions occur.For example,the
		/// current account will be frozen(if account owner set this action) when transfer amount is
		/// more than your transfer amount limitation
		#[pallet::weight(10_000)]
		pub fn set_risk_management(
			origin: OriginFor<T>,
			risk_management: RiskManagement,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match risk_management {
				RiskManagement::TimeFreeze(_freeze_time) => {
					ensure!(
						!RiskManagementOwner::<T>::contains_key(&who, 1),
						Error::<T>::FreezeTimeHasSet
					);
					RiskManagementOwner::<T>::insert(&who, 1, risk_management.clone());
					MapFreezeAccountTemporary::<T>::insert(
						&who,
						(frame_system::Pallet::<T>::block_number(), risk_management.clone()),
					);

					Self::deposit_event(Event::RiskManagementTimeFreezeSet(
						who.clone(),
						risk_management.clone(),
					));
					log::info!("--------------------------------set freeze time");
				},
				RiskManagement::AccountFreeze(_is_freeze) => {
					ensure!(
						!RiskManagementOwner::<T>::contains_key(&who, 2),
						Error::<T>::FreezeAccountHasSet
					);
					RiskManagementOwner::<T>::insert(&who, 2, risk_management.clone());

					Self::deposit_event(Event::RiskManagementAccountFreezeSet(
						who.clone(),
						risk_management.clone(),
					));
					log::info!("--------------------------------set freeze account");
				},
			}
			Ok(())
		}

		/// a function that is more safe when transfer money to others, it will check all conditions
		/// account owner set before when some illegal transactions triggered, account's balance
		/// will be protected and account owner will get notification timely via email,slack and
		/// discord
		#[pallet::weight(10_000)]
		pub fn safe_transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			value: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut account_is_frozen = false;
			let mut time_is_frozen = false;
			let mut default_freeze_status = false;

			let next_notify_account_index = NextNotifyAccountIndex::<T>::get().unwrap_or_default();

			let result = Self::check_account_status(who.clone());

			match result {
				1 => ensure!(result != 1, Error::<T>::AccountHasBeenFrozenForever),
				2 => ensure!(result != 2, Error::<T>::AccountHasBeenFrozenTemporary),
				_ => {},
			}

			if T::PermissionCaptureInterface::is_account_permission_taken(who.clone()) {
				let data = T::CustomCallInterface::call_transfer(to.clone(), value);

				let call_wrapper = OpaqueCall::<T>::from_encoded(data.clone());
				let call_hash = blake2_256(&data);

				if T::PermissionCaptureInterface::has_account_pedding_call(who.clone()) {
					if T::PermissionCaptureInterface::is_the_same_hash(who.clone(), call_hash) {
						ensure!(
							T::PermissionCaptureInterface::is_call_approved(who.clone(), call_hash),
							Error::<T>::PermissionTakenAccountCallMustBeApproved
						);
					} else {
						// Error: Already has a pedding call
						fail!(Error::<T>::PermissionTakenAccountHasPaddingCall)
					}
				} else {
					T::PermissionCaptureInterface::add_call_to_approval_list(
						who.clone(),
						call_hash,
						call_wrapper,
						Default::default(),
					);
					return Ok(());
				}
			}

			if let Some(TransferLimit::AmountLimit(amount)) = TransferLimitOwner::<T>::get(&who, 1)
			{
				if value > amount {
					let _account_status = Self::check_risk_management_config(who.clone());
					Self::set_notify_method(who.clone(), next_notify_account_index);

					// if !account_status {
					// 	ensure!(value < amount, Error::<T>::TransferValueTooLarge);
					// }
				}
			}

			if let Some(TransferLimit::FrequencyLimit(frequency, block_numbers)) =
				TransferLimitOwner::<T>::get(&who, 2)
			{
				if frequency <= 0 {
					let _account_status = Self::check_risk_management_config(who.clone());
					Self::set_notify_method(who.clone(), next_notify_account_index);

				// if !account_status {
				// 	ensure!(frequency > 0, Error::<T>::TransferTimesTooMany);
				// }
				} else {
					if let Some((block_number, TransferLimit::FrequencyLimit(fre, blocks))) =
						MapFrequencyLimit::<T>::get(who.clone())
					{
						let now = frame_system::Pallet::<T>::block_number();

						if now.saturated_into::<u64>()
							> blocks + block_number.saturated_into::<u64>()
						{
							Self::reset_frequency_limit_config(who.clone(), fre - 1);
						} else {
							TransferLimitOwner::<T>::mutate(&who, 2, |v| {
								*v = Some(TransferLimit::FrequencyLimit(
									frequency - 1,
									block_numbers,
								))
							});
						}
					}
				}
			}

			if let Some(val) = BlockAccount::<T>::get(who.clone()) {
				if val == true {
					account_is_frozen = true;
					log::info!(
						"-------------------------------check block account again {:?}",
						account_is_frozen
					);
				}
			}
			if let Some(val) = BlockTime::<T>::get(who.clone()) {
				if val == true {
					time_is_frozen = true;
					log::info!(
						"-------------------------------check block time again {:?} ",
						time_is_frozen
					);
				}
			}
			if let Some(val) = DefaultFreeze::<T>::get(who.clone()) {
				if val == true {
					default_freeze_status = true;
					log::info!(
						"-------------------------------check default freeze again {:?} ",
						default_freeze_status
					);
				}
			}

			if account_is_frozen || time_is_frozen || default_freeze_status {
				log::info!("-------------------------------transfer failed");
			} else {
				T::Currency::transfer(&who, &to, value, ExistenceRequirement::AllowDeath)?;
				Self::deposit_event(Event::TransferSuccess(who.clone(), to.clone(), value));
				log::info!("-------------------------------transfer successfully");
			}

			Ok(())
		}

		/// a function to allow user to freeze his account directly without
		/// when account is frozen, all transaction in the future will fail
		/// account owner needs to recover this account

		#[pallet::weight(10_000)]
		pub fn freeze_account(origin: OriginFor<T>, freeze: bool) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if freeze {
				Self::freeze_account_forever(who.clone());
				Self::deposit_event(Event::FreezeAccountSuccess(who.clone()));
			}

			Ok(())
		}

		/// notification will be send only one time
		/// when sending finished, resending flag will be set false

		#[pallet::weight(0)]
		pub fn reset_notification_status(
			origin: OriginFor<T>,
			account: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			if let Some(val) = MailStatus::<T>::get(account.clone()) {
				if val == true {
					MailStatus::<T>::mutate(account.clone(), |v| *v = Some(false));
					log::info!("-------------------------------- deactivate email notification");
				}
			}
			if let Some(val) = MailStatus::<T>::get(account.clone()) {
				if val == true {
					DiscordStatus::<T>::mutate(account.clone(), |v| *v = Some(false));
					log::info!("-------------------------------- deactivate discord notification");
				}
			}

			if let Some(val) = MailStatus::<T>::get(account.clone()) {
				if val == true {
					SlackStatus::<T>::mutate(account.clone(), |v| *v = Some(false));
					log::info!("-------------------------------- deactivate slack notification");
				}
			}

			let next_notify_account_index = NextNotifyAccountIndex::<T>::get().unwrap_or_default();

			for index in 0..next_notify_account_index {
				if let Some(val) = MapNotifyAccount::<T>::get(index) {
					if val == account {
						MapNotifyAccount::<T>::remove(index);
						log::info!("-------------------------------- remove the account which has been notified");
					}
				}
			}

			Ok(().into())
		}
	}

	/// all notification will be send via offchain_worker, it is more efficient
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("Hi World from defense-pallet workers!: {:?}", block_number);
			let next_notify_account_index = NextNotifyAccountIndex::<T>::get().unwrap_or_default();

			for i in 0..next_notify_account_index {
				match MapNotifyAccount::<T>::get(i) {
					Some(account) => Self::check_notify_method_and_send_info(account.clone()),
					_ => {},
				}
			}
		}
	}

	/// send email information
	/// all info will be send to mail server, then mail server will delivery mails to different to
	/// account owner
	impl<T: Config> Pallet<T> {
		fn send_email_info(account: T::AccountId) -> Result<u64, http::Error> {
			match T::Notification::get_mail_config_action(account) {
				Some(Action::MailWithToken(_, _, receiver, title, message_body)) => {
					let to_email =
						match scale_info::prelude::string::String::from_utf8(receiver.to_vec()) {
							Ok(v) => v,
							Err(e) => {
								log::info!("------decode receiver error  {:?}", e);
								"".to_string()
							},
						};

					let subject =
						match scale_info::prelude::string::String::from_utf8(title.to_vec()) {
							Ok(v) => v,
							Err(e) => {
								log::info!("------decode title error  {:?}", e);
								"".to_string()
							},
						};

					let content =
						match scale_info::prelude::string::String::from_utf8(message_body.to_vec())
						{
							Ok(v) => v,
							Err(e) => {
								log::info!("------decode message body error  {:?}", e);
								"".to_string()
							},
						};

					let basic_url = "http://127.0.0.1:3030/get?";

					let email = BASE64.encode(to_email[..].as_bytes());
					let subject = BASE64.encode(subject[..].as_bytes());
					let content = BASE64.encode(content[..].as_bytes());

					let url = &scale_info::prelude::format!(
						"{}email={}&subject={}&content={}",
						basic_url,
						email,
						subject,
						content
					)[..];

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
						return Err(http::Error::Unknown);
					} else {
						log::info!("email send successfully")
					}
					let body = response.body().collect::<Vec<u8>>();
					let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
						log::warn!("No UTF8 body");
						http::Error::Unknown
					})?;

					log::info!("get return value: {}", body_str);
				},
				_ => {},
			}

			Ok(0)
		}

		/// send discord information
		/// another way to get information for account owner
		fn send_discord_info(account: T::AccountId) -> Result<u64, http::Error> {
			if let Some(Action::Discord(discord_hook_url, username, content)) =
				T::Notification::get_discord_config_action(account)
			{
				let hook_url =
					match scale_info::prelude::string::String::from_utf8(discord_hook_url.to_vec())
					{
						Ok(v) => v,
						Err(e) => {
							log::info!("------decode receiver error  {:?}", e);
							"".to_string()
						},
					};

				let name = match scale_info::prelude::string::String::from_utf8(username.to_vec()) {
					Ok(v) => v,
					Err(e) => {
						log::info!("------decode title error  {:?}", e);
						"".to_string()
					},
				};

				let message = match scale_info::prelude::string::String::from_utf8(content.to_vec())
				{
					Ok(v) => v,
					Err(e) => {
						log::info!("------decode message body error  {:?}", e);
						"".to_string()
					},
				};

				let basic_url = "http://127.0.0.1:3030/get?";

				let discord_hook_url = BASE64.encode(hook_url[..].as_bytes());
				let discord_user_name = BASE64.encode(name[..].as_bytes());
				let discord_message = BASE64.encode(message[..].as_bytes());

				let url = &scale_info::prelude::format!(
					"{}email={}&subject={}&content={}",
					basic_url,
					discord_hook_url,
					discord_user_name,
					discord_message
				)[..];

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
					return Err(http::Error::Unknown);
				} else {
					log::info!("email send successfully")
				}
				let body = response.body().collect::<Vec<u8>>();
				let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
					log::warn!("No UTF8 body");
					http::Error::Unknown
				})?;

				log::info!("get return value: {}", body_str);
			}
			Ok(0)
		}
		/// send slack information
		/// another way to get information for account owner
		fn send_slack_info(account: T::AccountId) -> Result<u64, http::Error> {
			if let Some(Action::Slack(slack_hook_url, content)) =
				T::Notification::get_slack_config_action(account)
			{
				let hook_url =
					match scale_info::prelude::string::String::from_utf8(slack_hook_url.to_vec()) {
						Ok(v) => v,
						Err(e) => {
							log::info!("------decode receiver error  {:?}", e);
							"".to_string()
						},
					};

				let message = match scale_info::prelude::string::String::from_utf8(content.to_vec())
				{
					Ok(v) => v,
					Err(e) => {
						log::info!("------decode message body error  {:?}", e);
						"".to_string()
					},
				};

				let basic_url = "http://127.0.0.1:3030/get?";

				let slack_hook_url = BASE64.encode(hook_url[..].as_bytes());
				let slack_message = BASE64.encode(message[..].as_bytes());

				let url = &scale_info::prelude::format!(
					"{}email={}&subject={}",
					basic_url,
					slack_hook_url,
					slack_message
				)[..];

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
					return Err(http::Error::Unknown);
				} else {
					log::info!("email send successfully")
				}
				let body = response.body().collect::<Vec<u8>>();
				let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
					log::warn!("No UTF8 body");
					http::Error::Unknown
				})?;

				log::info!("get return value: {}", body_str);
			}
			Ok(0)
		}

		/// to change notification status so that void send info again
		fn send_signed_tx(account: T::AccountId) -> Result<(), &'static str> {
			let signer = Signer::<T, T::AuthorityId>::all_accounts();
			if !signer.can_sign() {
				return Err(
					"No local accounts available. Consider adding one via `author_insertKey` RPC.",
				);
			}

			let results = signer.send_signed_transaction(|_account| {
				Call::reset_notification_status { account: account.clone() }
			});

			for (acc, res) in &results {
				match res {
					Ok(()) => log::info!("[{:?}] Submitted change info", acc.id),
					Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
				}
			}

			Ok(())
		}

		/// a helper function to freeze account according to different protect actions
		fn check_risk_management_config(who: T::AccountId) -> bool {
			let mut account_freeze_status = false;

			if let Some(RiskManagement::AccountFreeze(account_need_freeze)) =
				RiskManagementOwner::<T>::get(&who, 2)
			{
				if account_need_freeze {
					Self::freeze_account_forever(who.clone());
					account_freeze_status = true;
				}
			}
			if account_freeze_status == false {
				if let Some(freeze_time_config) = RiskManagementOwner::<T>::get(&who, 1) {
					Self::freeze_account_temporary(who.clone(), freeze_time_config);
					account_freeze_status = true;
				}
			}

			if account_freeze_status == false {
				Self::default_freeze_account(who.clone());
				account_freeze_status = true;
			}

			account_freeze_status
		}

		fn freeze_account_forever(who: T::AccountId) {
			BlockAccount::<T>::insert(who.clone(), true);

			Self::deposit_event(Event::FreezeAccountForever(who.clone()));
			log::info!("--------------------------------freeze account forever");
		}

		fn freeze_account_temporary(who: T::AccountId, freeze_time_config: RiskManagement) {
			match BlockTime::<T>::get(&who) {
				Some(account_status) => {
					if account_status == false {
						BlockTime::<T>::mutate(who.clone(), |v| *v = Some(true));
					}
				},
				None => {
					BlockTime::<T>::insert(who.clone(), true);
				},
			}

			Self::deposit_event(Event::FreezeAccountTemporary(who.clone()));
			log::info!("--------------------------------freeze account temporary");
			Self::update_freeze_start_time(who.clone(), freeze_time_config)
		}

		fn default_freeze_account(who: T::AccountId) {
			match DefaultFreeze::<T>::get(who.clone()) {
				Some(_val) => DefaultFreeze::<T>::mutate(&who, |v| *v = Some(true)),
				None => {
					DefaultFreeze::<T>::insert(&who, true);
				},
			}
			match DefaultFreezeAccountTemporary::<T>::get(who.clone()) {
				Some(_) => DefaultFreezeAccountTemporary::<T>::mutate(&who, |v| {
					*v = Some((frame_system::Pallet::<T>::block_number(), 100));
				}),
				None => DefaultFreezeAccountTemporary::<T>::insert(
					&who,
					(frame_system::Pallet::<T>::block_number(), 100),
				),
			}
		}

		fn update_freeze_start_time(who: T::AccountId, freeze_time_config: RiskManagement) {
			// reset freeze start time
			MapFreezeAccountTemporary::<T>::mutate(&who, |v| {
				*v = Some((frame_system::Pallet::<T>::block_number(), freeze_time_config.clone()))
			});
		}

		/// set notify method such as email,discord and slack

		fn set_notify_method(who: T::AccountId, next_notify_account_index: u64) {
			match T::Notification::get_mail_config_action(who.clone()) {
				Some(Action::MailWithToken(..)) => {
					log::info!("--------------------------------mail with token status is none,set mail status true");
					MailStatus::<T>::insert(who.clone(), true);
					MapNotifyAccount::<T>::insert(next_notify_account_index, who.clone());
					NextNotifyAccountIndex::<T>::put(
						next_notify_account_index.saturating_add(One::one()),
					);
				},
				_ => {
					log::info!("--------------------------------no email");
				},
			}
			match T::Notification::get_slack_config_action(who.clone()) {
				Some(_) => {
					log::info!("-------------------------------- slack status is none,set mail status true");
					SlackStatus::<T>::insert(who.clone(), true);
					MapNotifyAccount::<T>::insert(next_notify_account_index, who.clone());
					NextNotifyAccountIndex::<T>::put(
						next_notify_account_index.saturating_add(One::one()),
					);
				},
				_ => {
					log::info!("--------------------------------no slack");
				},
			}
			match T::Notification::get_discord_config_action(who.clone()) {
				Some(_) => {
					log::info!("--------------------------------discord with token status is none,set mail status true");
					DiscordStatus::<T>::insert(who.clone(), true);
					MapNotifyAccount::<T>::insert(next_notify_account_index, who.clone());
					NextNotifyAccountIndex::<T>::put(
						next_notify_account_index.saturating_add(One::one()),
					);
				},
				_ => {
					log::info!("--------------------------------no discord");
				},
			}
		}

		/// check notify method and send info
		fn check_notify_method_and_send_info(account: T::AccountId) {
			if MailStatus::<T>::get(account.clone()) == Some(true) {
				match Self::send_email_info(account.clone()) {
					Ok(val) => {
						log::info!("email send successfully {:?}", val);
						match Self::send_signed_tx(account.clone()) {
							Ok(_) => {
								log::info!("reset notification status as false")
							},
							Err(e) => {
								log::info!("reset notification status as false failed {:?}", e);
							},
						};
					},
					Err(e) => log::info!("email send failed {:?}", e),
				}
			} else if DiscordStatus::<T>::get(account.clone()) == Some(true) {
				match Self::send_discord_info(account.clone()) {
					Ok(val) => {
						log::info!("email send successfully {:?}", val);
						match Self::send_signed_tx(account.clone()) {
							Ok(_) => {
								log::info!("reset notification status as false")
							},
							Err(e) => {
								log::info!("reset notification status as false failed {:?}", e);
							},
						};
					},
					Err(e) => log::info!("discord send failed {:?}", e),
				}
			} else if SlackStatus::<T>::get(account.clone()) == Some(true) {
				match Self::send_slack_info(account.clone()) {
					Ok(val) => {
						log::info!("email send successfully {:?}", val);
						match Self::send_signed_tx(account.clone()) {
							Ok(_) => {
								log::info!("reset notification status as false")
							},
							Err(e) => {
								log::info!("reset notification status as false failed {:?}", e);
							},
						};
					},
					Err(e) => log::info!("slack send failed {:?}", e),
				}
			} else {
				log::info!("no notify method available")
			}
		}

		/// check account status, if it's frozen forever, do nothing
		/// if it's frozen temporarily,keep still or unfrozen account
		fn check_account_status(who: T::AccountId) -> u32 {
			let mut account_status: u32 = 0;

			if let Some(val) = BlockAccount::<T>::get(who.clone()) {
				if val == true {
					log::info!("--------------------------------account has been freezed forever");
					account_status = 1;
				}
			}

			if account_status == 0 {
				if let Some(val) = BlockTime::<T>::get(who.clone()) {
					if val == true {
						if Self::check_amount_condition(who.clone()) {
							Self::unfreeze_account_temporary(who.clone());
							log::info!("--------------------------------account has been unfrozen");
						} else {
							account_status = 2;
							log::info!(
								"--------------------------------account has been frozen temporary"
							);
						}
					}
				}
			}
			if account_status == 0 {
				if let Some(val) = DefaultFreeze::<T>::get(who.clone()) {
					if val == true {
						if let Some((block_number, blocks)) =
							DefaultFreezeAccountTemporary::<T>::get(who.clone())
						{
							let now = frame_system::Pallet::<T>::block_number();

							if now.saturated_into::<u64>()
								> blocks + block_number.saturated_into::<u64>()
							{
								Self::unfreeze_account_temporary_default(who.clone());
							} else {
								account_status = 2;
								log::info!(
								"--------------------------------account has been frozen default"
							);
							}
						}
					}
				}
			}
			account_status
		}

		fn unfreeze_account_temporary(who: T::AccountId) {
			BlockTime::<T>::mutate(&who, |v| *v = Some(false));
		}

		fn unfreeze_account_temporary_default(who: T::AccountId) {
			DefaultFreeze::<T>::mutate(&who, |v| *v = Some(false));
		}

		/// a function to restore frequency limit config, including available frequency and start block height
		fn reset_frequency_limit_config(who: T::AccountId, frequency: u64) {
			if let Some((
				_times_limit_block_number,
				TransferLimit::FrequencyLimit(_frequency, block_numbers),
			)) = MapFrequencyLimit::<T>::get(&who)
			{
				// reset available frequency
				TransferLimitOwner::<T>::mutate(&who, 2, |v| {
					*v = Some(TransferLimit::FrequencyLimit(frequency, block_numbers))
				});

				// reset start block numbers
				MapFrequencyLimit::<T>::mutate(&who, |v| {
					*v = Some((
						frame_system::Pallet::<T>::block_number(),
						TransferLimit::FrequencyLimit(frequency, block_numbers).clone(),
					))
				});
			}
		}

		/// a function to check account status, when account satisfies the condition od releasing, return true
		fn check_amount_condition(who: T::AccountId) -> bool {
			let mut account_should_unfreeze = false;
			match MapFreezeAccountTemporary::<T>::get(&who) {
				Some((block_number, risk_management)) => {
					if let RiskManagement::TimeFreeze(freeze_time) = risk_management {
						let now = frame_system::Pallet::<T>::block_number();

						if now.saturated_into::<u64>()
							> freeze_time + block_number.saturated_into::<u64>()
						{
							account_should_unfreeze = true;
						} else {
							log::info!(
								"-------------------------------- freeze account will at {:?}",
								freeze_time + block_number.saturated_into::<u64>()
							);
						}
					}
				},
				None => {
					account_should_unfreeze = true;
				},
			}

			account_should_unfreeze
		}
	}
}
