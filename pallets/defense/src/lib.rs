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

use pallet_difttt::Action;

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

/// an enum, each instance represents a different transfer limit,
/// which is a single transfer amount limit and transfer frequency limit within a specified time
#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum TransferLimitation<Balance> {
	AmountLimit(Balance), // amount limit per transaction
	FrequencyLimit(u64, u64), /* the first parameter is times limit, the second parameter is
	                       * blocks limit */
}

/// an enum, each instance represents a different freeze configuration when malicious transfers
/// occur: freeze the account temporarily or permanently

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum FreezeConfiguration {
	TimeFreeze(u64),     // freeze account temporary
	AccountFreeze(bool), // freeze account permanent
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum AbnormalDetectionAction {
	NotifyByEmail, // notify by email
	FreezeAccount, // freeze account permanent
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

	use frame_system::offchain::SendUnsignedTransaction;
	use primitives::{
		custom_call::CustomCallInterface, permission_capture::PermissionCaptureInterface,
	};
	use sp_io::hashing::blake2_256;

	use pallet_notification::NotificationInfoInterface;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct NotificationSentPayload<Public, AccountId> {
		pub account: AccountId,
		pub mail_is_send: bool,
		pub slack_is_send: bool,
		pub discord_is_send: bool,
		pub public: Public,
	}
	impl<T: SigningTypes> SignedPayload<T> for NotificationSentPayload<T::Public, T::AccountId> {
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
	pub(super) type MapFrequencyLimit<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		(T::BlockNumber, TransferLimitation<BalanceOf<T>>),
	>;

	/// store freeze configuration
	#[pallet::storage]
	#[pallet::getter(fn freeze_account_temporary_map)]
	pub(super) type MapFreezeAccountTemporary<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (T::BlockNumber, FreezeConfiguration)>;

	/// store default freeze configuration
	#[pallet::storage]
	#[pallet::getter(fn default_freeze_account_temporary_map)]
	pub(super) type DefaultFreezeAccountTemporary<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (T::BlockNumber, u64)>;

	/// store all transfer limitations of the account
	#[pallet::storage]
	#[pallet::getter(fn transfer_limit_owner)]
	pub type TransferLimitationOwner<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		u64,
		TransferLimitation<BalanceOf<T>>,
		OptionQuery,
	>;

	/// stored owner of the frozen configuration
	#[pallet::storage]
	#[pallet::getter(fn freeze_configuration_owner)]
	pub type FreezeConfigurationOwner<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		u64,
		FreezeConfiguration,
		OptionQuery,
	>;

	/// storage account permanent frozen state
	#[pallet::storage]
	#[pallet::getter(fn block_account)]
	pub type BlockAccount<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	/// storage account temporary frozen state
	#[pallet::storage]
	#[pallet::getter(fn block_time)]
	pub type BlockTime<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	/// store account default frozen status
	#[pallet::storage]
	#[pallet::getter(fn default_freeze)]
	pub type DefaultFreeze<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	#[pallet::storage]
	#[pallet::getter(fn next_account_index)]
	pub type NextNotifyAccountIndex<T: Config> = StorageValue<_, u64>;

	/// store the accounts that need to send notifications
	#[pallet::storage]
	#[pallet::getter(fn notify_account_map)]
	pub(super) type MapNotifyAccount<T: Config> = StorageMap<_, Twox64Concat, u64, T::AccountId>;

	/// store the accounts that need to send emails
	#[pallet::storage]
	#[pallet::getter(fn mail_status)]
	pub(super) type MailStatus<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	/// store the accounts that need to send slack infos
	#[pallet::storage]
	#[pallet::getter(fn slack_status)]
	pub(super) type SlackStatus<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	/// store the accounts that need to send discord infos
	#[pallet::storage]
	#[pallet::getter(fn discord_status)]
	pub(super) type DiscordStatus<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	/// store the account's abnormal detection status
	#[pallet::storage]
	#[pallet::getter(fn abnormal_detection_status)]
	pub(super) type AbnormalDetectionStatus<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, bool>;

	/// store the account's abnormal detection action
	#[pallet::storage]
	#[pallet::getter(fn map_abnormal_detection_action)]
	pub(super) type MapAbnormalDetectionAction<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, AbnormalDetectionAction>;

	/// event emitted when the user performs an action successfully
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TransferAmountLimitSet(T::AccountId, TransferLimitation<BalanceOf<T>>), /* set amount
		                                                                         * limit */
		TransferFrequencyLimitSet(T::AccountId, TransferLimitation<BalanceOf<T>>), /* set frequency limit */
		FreezeTimeSet(T::AccountId, FreezeConfiguration), // set freeze account temporary
		FreezeAccountSet(T::AccountId, FreezeConfiguration), // set freeze account permanent
		FreezeAccountPermanent(T::AccountId),             // freeze account permanent
		FreezeAccountTemporary(T::AccountId),             // freeze account temporary
		TransferSuccess(T::AccountId, T::AccountId, BalanceOf<T>), // transfer success
		RemoveAddressFromNotifyList(T::AccountId),
		UpdateAbnormalDetectionStatus(T::AccountId, bool),
		UpdateAbnormalDetectionAction(T::AccountId, AbnormalDetectionAction),
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
		TransferTimesTooMany, // transfer frequency reach out transfer times limitation
		AccountHasBeenFrozenPermanent, // transfer again when account has been frozen permanent
		AccountHasBeenFrozenTemporary, // transfer again when account has been frozen temporary
		PermissionTakenAccountHasPaddingCall, // frozen account has been in pending list
		PermissionTakenAccountCallMustBeApproved, // take frozen account must be approved
		EmailConfigNotSet,    // email not set in pallet notification
	}

	/// a function can set different transfer limitations, such as amount per transaction, times per
	/// 100 blocks. All these limitations will be triggered when someone try to stole money from
	/// current account. at the same time, transaction will fail
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn set_transfer_limitation(
			origin: OriginFor<T>,
			transfer_limit: TransferLimitation<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match transfer_limit {
				TransferLimitation::AmountLimit(_amount) =>
					if TransferLimitationOwner::<T>::contains_key(&who, 1) == true {
						ensure!(
							!TransferLimitationOwner::<T>::contains_key(&who, 1),
							Error::<T>::TransferAmountLimitHasSet
						);
					} else {
						TransferLimitationOwner::<T>::insert(&who, 1, transfer_limit.clone());
						Self::deposit_event(Event::TransferAmountLimitSet(
							who.clone(),
							transfer_limit,
						));
						log::info!("--------------------------------set transfer amount limit");
					},
				TransferLimitation::FrequencyLimit(_max_available_amount, _blocks_limit) =>
					if TransferLimitationOwner::<T>::contains_key(&who, 2) == true {
						ensure!(
							!TransferLimitationOwner::<T>::contains_key(&who, 2),
							Error::<T>::TransferFrequencyLimitHasSet
						)
					} else {
						TransferLimitationOwner::<T>::insert(&who, 2, transfer_limit.clone());
						MapFrequencyLimit::<T>::insert(
							&who,
							(frame_system::Pallet::<T>::block_number(), transfer_limit.clone()),
						);
						Self::deposit_event(Event::TransferFrequencyLimitSet(
							who.clone(),
							transfer_limit,
						));

						log::info!("--------------------------------set transfer times limit");
					},
			}
			Ok(())
		}

		/// a function to set freeze configurations when illegal transactions occur.For example,the
		/// current account will be frozen when transfer amount is more than transfer amount
		/// limitation
		#[pallet::weight(10_000)]
		pub fn set_freeze_configuration(
			origin: OriginFor<T>,
			freeze_configuration: FreezeConfiguration,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match freeze_configuration {
				FreezeConfiguration::TimeFreeze(_freeze_time) => {
					ensure!(
						!FreezeConfigurationOwner::<T>::contains_key(&who, 1),
						Error::<T>::FreezeTimeHasSet
					);
					FreezeConfigurationOwner::<T>::insert(&who, 1, freeze_configuration.clone());
					MapFreezeAccountTemporary::<T>::insert(
						&who,
						(frame_system::Pallet::<T>::block_number(), freeze_configuration.clone()),
					);

					Self::deposit_event(Event::FreezeTimeSet(
						who.clone(),
						freeze_configuration.clone(),
					));
					log::info!("--------------------------------set freeze time");
				},
				FreezeConfiguration::AccountFreeze(_is_freeze) => {
					ensure!(
						!FreezeConfigurationOwner::<T>::contains_key(&who, 2),
						Error::<T>::FreezeAccountHasSet
					);
					FreezeConfigurationOwner::<T>::insert(&who, 2, freeze_configuration.clone());

					Self::deposit_event(Event::FreezeAccountSet(
						who.clone(),
						freeze_configuration.clone(),
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
				1 => ensure!(result != 1, Error::<T>::AccountHasBeenFrozenPermanent),
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
						// Error: Already has a pending call
						fail!(Error::<T>::PermissionTakenAccountHasPaddingCall)
					}
				} else {
					T::PermissionCaptureInterface::add_call_to_approval_list(
						who.clone(),
						call_hash,
						call_wrapper,
						Default::default(),
					);
					return Ok(())
				}
			}

			if let Some(TransferLimitation::AmountLimit(amount)) =
				TransferLimitationOwner::<T>::get(&who, 1)
			{
				if value > amount {
					let _account_status = Self::check_freeze_configuration_config(who.clone());
					Self::set_notify_method(who.clone(), next_notify_account_index);
				}
			}

			if let Some(TransferLimitation::FrequencyLimit(frequency, block_numbers)) =
				TransferLimitationOwner::<T>::get(&who, 2)
			{
				if frequency <= 0 {
					let _account_status = Self::check_freeze_configuration_config(who.clone());
					Self::set_notify_method(who.clone(), next_notify_account_index);
				} else {
					if let Some((block_number, TransferLimitation::FrequencyLimit(fre, blocks))) =
						MapFrequencyLimit::<T>::get(who.clone())
					{
						let now = frame_system::Pallet::<T>::block_number();

						if now.saturated_into::<u64>() >
							blocks + block_number.saturated_into::<u64>()
						{
							Self::reset_frequency_limit_config(who.clone(), fre - 1);
						} else {
							TransferLimitationOwner::<T>::mutate(&who, 2, |v| {
								*v = Some(TransferLimitation::FrequencyLimit(
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

		/// a function to allow user to freeze his account directly
		/// when account is frozen, all transaction in the future will fail
		/// account owner needs to recover this account via pallet-permission_capture

		#[pallet::weight(10_000)]
		pub fn freeze_account(origin: OriginFor<T>, freeze: bool) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if freeze {
				Self::freeze_account_permanent(who.clone());
				Self::deposit_event(Event::FreezeAccountPermanent(who.clone()));
			}

			Ok(())
		}

		/// notification will be send only one time
		/// when sending finished, resending flag will be set false
		#[pallet::weight(0)]
		pub fn deactivate_notification(
			origin: OriginFor<T>,
			notification_sent_payload: NotificationSentPayload<T::Public, T::AccountId>,
			_signature: T::Signature,
		) -> DispatchResultWithPostInfo {
			let _who = ensure_none(origin)?;

			log::info!(
				"-------------------------------- get payload {:?}",
				notification_sent_payload
			);

			if notification_sent_payload.mail_is_send {
				if let Some(val) = MailStatus::<T>::get(notification_sent_payload.account.clone()) {
					if val == true {
						MailStatus::<T>::mutate(notification_sent_payload.account.clone(), |v| {
							*v = Some(false)
						});
						log::info!(
							"-------------------------------- deactivate email notification"
						);
					}
				}
			}

			if notification_sent_payload.slack_is_send {
				if let Some(val) = SlackStatus::<T>::get(notification_sent_payload.account.clone())
				{
					if val == true {
						SlackStatus::<T>::mutate(notification_sent_payload.account.clone(), |v| {
							*v = Some(false)
						});

						log::info!(
							"-------------------------------- deactivate slack notification"
						);
					}
				}
			}

			if notification_sent_payload.discord_is_send {
				if let Some(val) =
					DiscordStatus::<T>::get(notification_sent_payload.account.clone())
				{
					if val == true {
						DiscordStatus::<T>::mutate(
							notification_sent_payload.account.clone(),
							|v| *v = Some(false),
						);
						log::info!(
							"-------------------------------- deactivate discord notification"
						);
					}
				}
			}

			let mut mail_status = false;
			let mut slack_status = false;
			let mut discord_status = false;

			if let Some(val) = MailStatus::<T>::get(notification_sent_payload.account.clone()) {
				if val == true {
					mail_status = true;
				}
			}
			if let Some(val) = SlackStatus::<T>::get(notification_sent_payload.account.clone()) {
				if val == true {
					slack_status = true;
				}
			}
			if let Some(val) = DiscordStatus::<T>::get(notification_sent_payload.account.clone()) {
				if val == true {
					discord_status = true;
				}
			}

			if mail_status || slack_status || discord_status {
				log::info!(
					"--------------------------------  mail_status {:?},  slack_status {:?},discord_status {:?}",mail_status,slack_status,discord_status
				);
			} else {
				let next_notify_account_index =
					NextNotifyAccountIndex::<T>::get().unwrap_or_default();

				for index in 0..next_notify_account_index {
					if let Some(val) = MapNotifyAccount::<T>::get(index) {
						if val == notification_sent_payload.account {
							MapNotifyAccount::<T>::remove(index);
							Self::deposit_event(Event::RemoveAddressFromNotifyList(
								notification_sent_payload.account.clone(),
							));
							log::info!("-------------------------------- remove the account which has been notified");
						}
					}
				}
			}

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn set_abnormal_detection_status(
			origin: OriginFor<T>,
			enable: bool,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			AbnormalDetectionStatus::<T>::insert(who.clone(), enable);
			Self::deposit_event(Event::UpdateAbnormalDetectionStatus(who.clone(), enable));
		}

		#[pallet::weight(0)]
		pub fn set_abnormal_detection_action(
			origin: OriginFor<T>,
			action: AbnormalDetectionAction,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			match action {
				AbnormalDetectionAction::NotifyByEmail => {
					ensure!(
						T::Notification::get_mail_config_action(account).is_some(),
						Error::<T>::EmailConfigNotSet,
					);
				},
				_ => {},
			}

			MapAbnormalDetectionAction::<T>::insert(who.clone(), action);
			Self::deposit_event(Event::UpdateAbnormalDetectionAction(who.clone(), action));
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
					Some(account) => {
						let _res = Self::check_notify_method_and_send_info(account.clone());
					},
					_ => {},
				}
			}
		}
	}

	/// helper functions
	impl<T: Config> Pallet<T> {
		// send email information
		// all info will be send to mail server, then mail server will delivery mails to different
		// users
		fn send_email_info(account: T::AccountId) -> Result<u64, http::Error> {
			if let Some(val) = MailStatus::<T>::get(account.clone()) {
				if val == true {
					match T::Notification::get_mail_config_action(account) {
						Some(Action::MailWithToken(_, _, receiver, title, message_body)) => {
							let to_email = match scale_info::prelude::string::String::from_utf8(
								receiver.to_vec(),
							) {
								Ok(v) => v,
								Err(e) => {
									log::info!("------decode receiver error  {:?}", e);
									"".to_string()
								},
							};

							let subject = match scale_info::prelude::string::String::from_utf8(
								title.to_vec(),
							) {
								Ok(v) => v,
								Err(e) => {
									log::info!("------decode title error  {:?}", e);
									"".to_string()
								},
							};

							let content = match scale_info::prelude::string::String::from_utf8(
								message_body.to_vec(),
							) {
								Ok(v) => v,
								Err(e) => {
									log::info!("------decode message body error  {:?}", e);
									"".to_string()
								},
							};

							let basic_url = "http://127.0.0.1:3030/mail?";

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

							let deadline =
								sp_io::offchain::timestamp().add(Duration::from_millis(10_000));

							let request = http::Request::get(url);

							let pending = request.deadline(deadline).send().map_err(|e| {
								log::info!("---------get pending error: {:?}", e);
								http::Error::IoError
							})?;

							let response = pending
								.try_wait(deadline)
								.map_err(|_| http::Error::DeadlineReached)??;
							if response.code != 200 {
								log::warn!("Unexpected status code: {}", response.code);
								return Err(http::Error::Unknown)
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
				}
			}

			Ok(0)
		}

		// send discord information
		// another way to get information for account owner
		fn send_discord_info(account: T::AccountId) -> Result<u64, http::Error> {
			if let Some(val) = DiscordStatus::<T>::get(account.clone()) {
				if val == true {
					if let Some(Action::Discord(discord_hook_url, username, content)) =
						T::Notification::get_discord_config_action(account)
					{
						let hook_url = match scale_info::prelude::string::String::from_utf8(
							discord_hook_url.to_vec(),
						) {
							Ok(v) => v,
							Err(e) => {
								log::info!("------decode receiver error  {:?}", e);
								"".to_string()
							},
						};

						let name =
							match scale_info::prelude::string::String::from_utf8(username.to_vec())
							{
								Ok(v) => v,
								Err(e) => {
									log::info!("------decode title error  {:?}", e);
									"".to_string()
								},
							};

						let message = match scale_info::prelude::string::String::from_utf8(
							content.to_vec(),
						) {
							Ok(v) => v,
							Err(e) => {
								log::info!("------decode message body error  {:?}", e);
								"".to_string()
							},
						};

						let basic_url = "http://127.0.0.1:3030/discord?";

						let discord_hook_url = BASE64.encode(hook_url[..].as_bytes());
						let discord_user_name = BASE64.encode(name[..].as_bytes());
						let discord_message = BASE64.encode(message[..].as_bytes());

						let url = &scale_info::prelude::format!(
							"{}url={}&name={}&content={}",
							basic_url,
							discord_hook_url,
							discord_user_name,
							discord_message
						)[..];

						let deadline =
							sp_io::offchain::timestamp().add(Duration::from_millis(10_000));

						let request = http::Request::get(url);

						let pending = request.deadline(deadline).send().map_err(|e| {
							log::info!("---------get pending error: {:?}", e);
							http::Error::IoError
						})?;

						let response = pending
							.try_wait(deadline)
							.map_err(|_| http::Error::DeadlineReached)??;
						if response.code != 200 {
							log::warn!("Unexpected status code: {}", response.code);
							return Err(http::Error::Unknown)
						} else {
							log::info!("discord send successfully")
						}
						let body = response.body().collect::<Vec<u8>>();
						let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
							log::warn!("No UTF8 body");
							http::Error::Unknown
						})?;

						log::info!("get return value: {}", body_str);
					}
				}
			}

			Ok(0)
		}
		// send slack information
		// another way to get information for account owner
		fn send_slack_info(account: T::AccountId) -> Result<u64, http::Error> {
			if let Some(val) = SlackStatus::<T>::get(account.clone()) {
				if val == true {
					if let Some(Action::Slack(slack_hook_url, content)) =
						T::Notification::get_slack_config_action(account)
					{
						let hook_url = match scale_info::prelude::string::String::from_utf8(
							slack_hook_url.to_vec(),
						) {
							Ok(v) => v,
							Err(e) => {
								log::info!("------decode receiver error  {:?}", e);
								"".to_string()
							},
						};

						let message = match scale_info::prelude::string::String::from_utf8(
							content.to_vec(),
						) {
							Ok(v) => v,
							Err(e) => {
								log::info!("------decode message body error  {:?}", e);
								"".to_string()
							},
						};

						let slack_hook_url = BASE64.encode(hook_url[..].as_bytes());
						let slack_message = BASE64.encode(message[..].as_bytes());

						let basic_url = "http://127.0.0.1:3030/slack?";

						let url = &scale_info::prelude::format!(
							"{}url={}&content={}",
							basic_url,
							slack_hook_url,
							slack_message
						)[..];

						let deadline =
							sp_io::offchain::timestamp().add(Duration::from_millis(10_000));

						let request = http::Request::get(url);

						let pending = request.deadline(deadline).send().map_err(|e| {
							log::info!("---------get pending error: {:?}", e);
							http::Error::IoError
						})?;

						let response = pending
							.try_wait(deadline)
							.map_err(|_| http::Error::DeadlineReached)??;
						if response.code != 200 {
							log::warn!("Unexpected status code: {}", response.code);
							return Err(http::Error::Unknown)
						} else {
							log::info!("slack send successfully")
						}
						let body = response.body().collect::<Vec<u8>>();
						let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
							log::warn!("No UTF8 body");
							http::Error::Unknown
						})?;

						log::info!("get return value: {}", body_str);
					}
				}
			}

			Ok(0)
		}

		// a helper function to check freeze configuration
		fn check_freeze_configuration_config(who: T::AccountId) -> bool {
			let mut account_freeze_status = false;

			if let Some(FreezeConfiguration::AccountFreeze(account_need_freeze)) =
				FreezeConfigurationOwner::<T>::get(&who, 2)
			{
				if account_need_freeze {
					Self::freeze_account_permanent(who.clone());
					account_freeze_status = true;
				}
			}
			if account_freeze_status == false {
				if let Some(freeze_time_config) = FreezeConfigurationOwner::<T>::get(&who, 1) {
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

		// a helper function to freeze account permanently
		fn freeze_account_permanent(who: T::AccountId) {
			BlockAccount::<T>::insert(who.clone(), true);

			Self::deposit_event(Event::FreezeAccountPermanent(who.clone()));
			log::info!("--------------------------------freeze account permanent");
		}

		// a helper function to freeze account temporarily
		fn freeze_account_temporary(who: T::AccountId, freeze_time_config: FreezeConfiguration) {
			match BlockTime::<T>::get(&who) {
				Some(account_status) =>
					if account_status == false {
						BlockTime::<T>::mutate(who.clone(), |v| *v = Some(true));
					},
				None => {
					BlockTime::<T>::insert(who.clone(), true);
				},
			}

			Self::deposit_event(Event::FreezeAccountTemporary(who.clone()));
			log::info!("--------------------------------freeze account temporary");
			Self::update_freeze_start_time(who.clone(), freeze_time_config)
		}

		// a default helper function to freeze account temporarily when user without setting any
		// freeze configurations
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

		// a helper function to update start time of freezing account
		fn update_freeze_start_time(who: T::AccountId, freeze_time_config: FreezeConfiguration) {
			// reset freeze start time
			MapFreezeAccountTemporary::<T>::mutate(&who, |v| {
				*v = Some((frame_system::Pallet::<T>::block_number(), freeze_time_config.clone()))
			});
		}

		// a helper function to set notify method such as email,discord and slack
		fn set_notify_method(who: T::AccountId, next_notify_account_index: u64) {
			match T::Notification::get_mail_config_action(who.clone()) {
				Some(Action::MailWithToken(..)) => {
					log::info!("--------------------------------mail with token status is none,set mail status true");
					match MailStatus::<T>::get(&who) {
						Some(_) => {
							MailStatus::<T>::mutate(&who, |v| *v = Some(true));
							MapNotifyAccount::<T>::insert(next_notify_account_index, who.clone());
							NextNotifyAccountIndex::<T>::put(
								next_notify_account_index.saturating_add(One::one()),
							);
						},
						None => {
							MailStatus::<T>::insert(who.clone(), true);
							MapNotifyAccount::<T>::insert(next_notify_account_index, who.clone());
							NextNotifyAccountIndex::<T>::put(
								next_notify_account_index.saturating_add(One::one()),
							);
						},
					}
				},
				_ => {
					log::info!("--------------------------------no email");
				},
			}
			match T::Notification::get_slack_config_action(who.clone()) {
				Some(_) => {
					log::info!("--------------------------------mail with token status is none,set mail status true");
					match SlackStatus::<T>::get(&who) {
						Some(_) => {
							SlackStatus::<T>::mutate(&who, |v| *v = Some(true));
							MapNotifyAccount::<T>::insert(next_notify_account_index, who.clone());
							NextNotifyAccountIndex::<T>::put(
								next_notify_account_index.saturating_add(One::one()),
							);
						},
						None => {
							SlackStatus::<T>::insert(who.clone(), true);
							MapNotifyAccount::<T>::insert(next_notify_account_index, who.clone());
							NextNotifyAccountIndex::<T>::put(
								next_notify_account_index.saturating_add(One::one()),
							);
						},
					}
				},
				_ => {
					log::info!("--------------------------------no slack");
				},
			}
			match T::Notification::get_discord_config_action(who.clone()) {
				Some(_) => {
					log::info!("--------------------------------mail with token status is none,set mail status true");
					match DiscordStatus::<T>::get(&who) {
						Some(_) => {
							DiscordStatus::<T>::mutate(&who, |v| *v = Some(true));
							MapNotifyAccount::<T>::insert(next_notify_account_index, who.clone());
							NextNotifyAccountIndex::<T>::put(
								next_notify_account_index.saturating_add(One::one()),
							);
						},
						None => {
							DiscordStatus::<T>::insert(who.clone(), true);
							MapNotifyAccount::<T>::insert(next_notify_account_index, who.clone());
							NextNotifyAccountIndex::<T>::put(
								next_notify_account_index.saturating_add(One::one()),
							);
						},
					}
				},
				_ => {
					log::info!("--------------------------------no discord");
				},
			}
		}

		// a helper function to check whether need to notify and send info
		fn check_notify_method_and_send_info(who: T::AccountId) -> Result<u64, Error<T>> {
			let mut mail_is_send = false;
			let mut slack_is_send = false;
			let mut discord_is_send = false;
			if let Some(val) = MailStatus::<T>::get(who.clone()) {
				if val == true {
					match Self::send_email_info(who.clone()) {
						Ok(val) => {
							log::info!("email send successfully {:?}", val);
							mail_is_send = true;
						},
						Err(e) => log::info!("email send failed {:?}", e),
					}
				}
			}

			if let Some(val) = DiscordStatus::<T>::get(who.clone()) {
				if val == true {
					match Self::send_discord_info(who.clone()) {
						Ok(val) => {
							log::info!("discord send successfully {:?}", val);
							discord_is_send = true;
						},
						Err(e) => log::info!("discord send failed {:?}", e),
					}
				}
			}

			if let Some(val) = SlackStatus::<T>::get(who.clone()) {
				if val == true {
					match Self::send_slack_info(who.clone()) {
						Ok(val) => {
							log::info!("slack send successfully {:?}", val);
							slack_is_send = true;
						},
						Err(e) => log::info!("slack send failed {:?}", e),
					}
				}
			}

			if let Some((_, res)) = Signer::<T, T::AuthorityId>::any_account()
				.send_unsigned_transaction(
					// this line is to prepare and return payload
					|account| NotificationSentPayload {
						account: who.clone(),
						mail_is_send,
						slack_is_send,
						discord_is_send,
						public: account.public.clone(),
					},
					|payload, signature| Call::deactivate_notification {
						notification_sent_payload: payload,
						signature,
					},
				) {
				match res {
					Ok(()) => {
						log::info!("-----unsigned tx with signed payload successfully sent.");
					},
					Err(()) => {
						log::error!("---sending unsigned tx with signed payload failed.");
					},
				};
			} else {
				// The case of `None`: no account is available for sending
				log::error!("----No local account available");
			}

			Ok(0)
		}

		// a helper function to check account status, if it's frozen permanent, do nothing
		// if it's frozen temporarily,keep still or unfreeze account
		fn check_account_status(who: T::AccountId) -> u32 {
			let mut account_status: u32 = 0;

			if let Some(val) = BlockAccount::<T>::get(who.clone()) {
				if val == true {
					log::info!(
						"--------------------------------account has been freezed permanent"
					);
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

							if now.saturated_into::<u64>() >
								blocks + block_number.saturated_into::<u64>()
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

		// a helper function to unfreeze account when user set freeze account temporarily
		fn unfreeze_account_temporary(who: T::AccountId) {
			BlockTime::<T>::mutate(&who, |v| *v = Some(false));
		}

		// a helper function to unfreeze account when user doesn't set freeze account configuration
		fn unfreeze_account_temporary_default(who: T::AccountId) {
			DefaultFreeze::<T>::mutate(&who, |v| *v = Some(false));
		}

		// a helper function to restore frequency limit config, including available frequency and
		// start block height
		fn reset_frequency_limit_config(who: T::AccountId, frequency: u64) {
			if let Some((
				_times_limit_block_number,
				TransferLimitation::FrequencyLimit(_frequency, block_numbers),
			)) = MapFrequencyLimit::<T>::get(&who)
			{
				// reset available frequency
				TransferLimitationOwner::<T>::mutate(&who, 2, |v| {
					*v = Some(TransferLimitation::FrequencyLimit(frequency, block_numbers))
				});

				// reset start block numbers
				MapFrequencyLimit::<T>::mutate(&who, |v| {
					*v = Some((
						frame_system::Pallet::<T>::block_number(),
						TransferLimitation::FrequencyLimit(frequency, block_numbers).clone(),
					))
				});
			}
		}

		// a helper function to check account status, when account satisfies the condition,return
		// true
		fn check_amount_condition(who: T::AccountId) -> bool {
			let mut account_should_unfreeze = false;
			match MapFreezeAccountTemporary::<T>::get(&who) {
				Some((block_number, freeze_configuration)) =>
					if let FreezeConfiguration::TimeFreeze(freeze_time) = freeze_configuration {
						let now = frame_system::Pallet::<T>::block_number();

						if now.saturated_into::<u64>() >
							freeze_time + block_number.saturated_into::<u64>()
						{
							account_should_unfreeze = true;
						} else {
							log::info!(
								"-------------------------------- freeze account will at {:?}",
								freeze_time + block_number.saturated_into::<u64>()
							);
						}
					},
				None => {
					account_should_unfreeze = true;
				},
			}

			account_should_unfreeze
		}
	}

	/// configure unsigned tx, use it to update onchain status of notification, so that
	/// notifications will not send repeatedly
	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// if let Call::deactivate_notification {
			// 	account: _,
			// 	mail_send: _,
			// 	slack_send: _,
			// 	discord_send: _,
			// } = call
			// {
			// 	//let provide = b"submit_xxx_unsigned".to_vec();
			// 	ValidTransaction::with_tag_prefix("DeactivateNotification")
			// 		.priority(T::UnsignedPriority::get())
			// 		.and_provides(1)
			// 		.longevity(3)
			// 		.propagate(true)
			// 		.build()
			// } else {
			// 	InvalidTransaction::Call.into()
			// }

			let valid_tx = |provide| {
				ValidTransaction::with_tag_prefix("ocw-defense")
					.priority(T::UnsignedPriority::get())
					.and_provides([&provide])
					.longevity(3)
					.propagate(true)
					.build()
			};

			match call {
				Call::deactivate_notification {
					notification_sent_payload: ref payload,
					ref signature,
				} => {
					let signature_valid =
						SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
					if !signature_valid {
						return InvalidTransaction::BadProof.into()
					}

					valid_tx(b"deactivate_notification".to_vec())
				},
				_ => InvalidTransaction::Call.into(),
			}
		}
	}
}
