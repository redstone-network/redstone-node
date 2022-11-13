#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
use codec::{Decode, Encode, MaxEncodedLen};
use pallet_difttt::Action;
use scale_info::TypeInfo;
use sp_runtime::offchain::{http, Duration};
use sp_runtime::RuntimeDebug;
use sp_std::cmp::{Eq, PartialEq};

use frame_system::offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer};
use sp_core::crypto::KeyTypeId;

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

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum TransferLimit<Balance> {
	AmountLimit(u64, Balance), // limit per transaction
	TimesLimit(u64, u64),      // limit on transactions per 100 blocks
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum RiskManagement {
	TimeFreeze(u64, u64), // freeze duration
	AccountFreeze(bool),
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::ReservableCurrency;
	use frame_support::traits::{Currency, ExistenceRequirement};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::One;
	use sp_runtime::traits::SaturatedConversion;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	use codec::alloc::string::ToString;
	use data_encoding::BASE64;
	use frame_support::inherent::Vec;
	use pallet_notification::NotificationInfoInterface;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
		type Notification: NotificationInfoInterface<Self::AccountId, Action<Self::AccountId>>;
	}

	/// store transfer limit
	#[pallet::storage]
	#[pallet::getter(fn transfer_limit_map)]
	pub(super) type MapTransferLimit<T: Config> =
		StorageMap<_, Twox64Concat, u64, (T::BlockNumber, TransferLimit<BalanceOf<T>>)>;

	/// store risk management
	#[pallet::storage]
	#[pallet::getter(fn risk_management_map)]
	pub(super) type MapRiskManagement<T: Config> =
		StorageMap<_, Twox64Concat, u64, (T::BlockNumber, RiskManagement)>;

	/// storage transfer limit owner
	#[pallet::storage]
	#[pallet::getter(fn transfer_limit_owner)]
	pub type TransferLimitOwner<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u64, (), OptionQuery>;

	/// store risk management owner
	#[pallet::storage]
	#[pallet::getter(fn risk_management_owner)]
	pub type RiskManagementOwner<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u64, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_transfer_limit_id)]
	pub type NextTransferLimitId<T: Config> = StorageValue<_, u64>;

	#[pallet::storage]
	#[pallet::getter(fn next_risk_management_id)]
	pub type NextRiskManagementId<T: Config> = StorageValue<_, u64>;

	#[pallet::storage]
	#[pallet::getter(fn block_account)]
	pub type BlockAccount<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	#[pallet::storage]
	#[pallet::getter(fn block_time)]
	pub type BlockTime<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	#[pallet::storage]
	#[pallet::getter(fn next_account_index)]
	pub type NextNotifyAccountIndex<T: Config> = StorageValue<_, u64>;

	/// store risk management
	#[pallet::storage]
	#[pallet::getter(fn notify_account_map)]
	pub(super) type MapNotifyAccount<T: Config> = StorageMap<_, Twox64Concat, u64, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn mail_status)]
	pub(super) type MailStatus<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	#[pallet::storage]
	#[pallet::getter(fn slack_status)]
	pub(super) type SlackStatus<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	#[pallet::storage]
	#[pallet::getter(fn discord_status)]
	pub(super) type DiscordStatus<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TransferAmountLimitSet(T::AccountId, TransferLimit<BalanceOf<T>>),
		TransferTimesLimitSet(T::AccountId, TransferLimit<BalanceOf<T>>),
		TransferAmountLimitUpdated(T::AccountId, TransferLimit<BalanceOf<T>>),
		TransferTimesLimitUpdated(T::AccountId, TransferLimit<BalanceOf<T>>),
		RiskManagementTimeFreezeSet(T::AccountId, RiskManagement),
		RiskManagementAccountFreezeSet(T::AccountId, RiskManagement),
		RiskManagementMailSet(T::AccountId, RiskManagement),
		RiskManagementMailUpdated(T::AccountId, RiskManagement),
		TransferSuccess(T::AccountId, T::AccountId, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		FreezeTimeHasSet,
		FreezeAccountHasSet,
		TransferValueTooLarge,
		TransferTimesTooMany,
		AccountHasBeenFrozenForever,
		AccountHasBeenFrozenTemporary,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn set_transfer_limit(
			origin: OriginFor<T>,
			transfer_limit: TransferLimit<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let transfer_limit_id = NextTransferLimitId::<T>::get().unwrap_or_default();

			let mut set_amount_limit = false;
			let mut set_times_limit = false;

			for i in 0..transfer_limit_id {
				if TransferLimitOwner::<T>::contains_key(&who, &i) == true {
					match transfer_limit {
						TransferLimit::AmountLimit(_set_time, _amount) => {
							if let Some((_, TransferLimit::AmountLimit(_set_time, _set_times))) =
								MapTransferLimit::<T>::get(i)
							{
								MapTransferLimit::<T>::mutate(&i, |v| {
									*v = Some((
										frame_system::Pallet::<T>::block_number(),
										transfer_limit.clone(),
									))
								});

								Self::deposit_event(Event::TransferAmountLimitUpdated(
									who.clone(),
									transfer_limit,
								));

								set_amount_limit = true;
								log::info!("--------------------------------update transfer amount limit successfully");

								break;
							}
						},
						TransferLimit::TimesLimit(_set_time, _times) => {
							if let Some((_, TransferLimit::TimesLimit(_set_time, _set_times))) =
								MapTransferLimit::<T>::get(i)
							{
								MapTransferLimit::<T>::mutate(&i, |v| {
									*v = Some((
										frame_system::Pallet::<T>::block_number(),
										transfer_limit.clone(),
									))
								});

								Self::deposit_event(Event::TransferTimesLimitUpdated(
									who.clone(),
									transfer_limit,
								));
								set_times_limit = true;
								log::info!("--------------------------------update transfer times limit successfully");

								break;
							}
						},
					}
				}
			}

			if set_times_limit == false && set_amount_limit == false {
				MapTransferLimit::<T>::insert(
					transfer_limit_id,
					(frame_system::Pallet::<T>::block_number(), transfer_limit.clone()),
				);
				TransferLimitOwner::<T>::insert(&who, transfer_limit_id, ());
				NextTransferLimitId::<T>::put(transfer_limit_id.saturating_add(One::one()));

				match transfer_limit {
					TransferLimit::AmountLimit(_set_time, _amount) => {
						log::info!("--------------------------------set transfer amount limit");
						Self::deposit_event(Event::TransferAmountLimitSet(
							who.clone(),
							transfer_limit,
						));
					},
					TransferLimit::TimesLimit(_set_time, _times) => {
						log::info!("--------------------------------set transfer times limit");
						Self::deposit_event(Event::TransferTimesLimitSet(
							who.clone(),
							transfer_limit,
						));
					},
				}
			}

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn set_risk_management(
			origin: OriginFor<T>,
			risk_management: RiskManagement,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let risk_management_id = NextRiskManagementId::<T>::get().unwrap_or_default();

			let mut freeze_time_set = false;
			let mut freeze_account_set = false;

			for i in 0..risk_management_id {
				if RiskManagementOwner::<T>::contains_key(&who, &i) == true {
					match risk_management {
						RiskManagement::TimeFreeze(_, _) => {
							if let Some((_, RiskManagement::TimeFreeze(_, _))) =
								MapRiskManagement::<T>::get(&i)
							{
								freeze_time_set = true;
								log::info!("-------------------------------- freeze time has set");
								ensure!(
									!MapRiskManagement::<T>::contains_key(&i),
									Error::<T>::FreezeTimeHasSet
								);
							}
						},
						RiskManagement::AccountFreeze(_) => {
							if let Some((_, RiskManagement::AccountFreeze(_))) =
								MapRiskManagement::<T>::get(&i)
							{
								freeze_account_set = true;
								log::info!(
									"-------------------------------- freeze account has set"
								);
								ensure!(
									!MapRiskManagement::<T>::contains_key(&i),
									Error::<T>::FreezeAccountHasSet
								);
							}
						},
					}
				}
			}

			if freeze_time_set == false && freeze_account_set == false {
				MapRiskManagement::<T>::insert(
					risk_management_id,
					(frame_system::Pallet::<T>::block_number(), risk_management.clone()),
				);
				RiskManagementOwner::<T>::insert(&who, risk_management_id, ());
				NextRiskManagementId::<T>::put(risk_management_id.saturating_add(One::one()));

				match risk_management {
					RiskManagement::TimeFreeze(_, _) => {
						log::info!("--------------------------------set freeze time");
						Self::deposit_event(Event::RiskManagementTimeFreezeSet(
							who.clone(),
							risk_management.clone(),
						));
					},
					RiskManagement::AccountFreeze(_) => {
						log::info!("--------------------------------set freeze account");
						Self::deposit_event(Event::RiskManagementAccountFreezeSet(
							who.clone(),
							risk_management.clone(),
						));
					},
				}
			}

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn safe_transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			value: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let transfer_limit_id = NextTransferLimitId::<T>::get().unwrap_or_default();
			let risk_management_id = NextRiskManagementId::<T>::get().unwrap_or_default();
			let next_notify_account_index = NextNotifyAccountIndex::<T>::get().unwrap_or_default();

			#[warn(unused_must_use)]
			Self::check_account_status(who.clone(), risk_management_id);

			if transfer_limit_id == 0 {
				log::info!("--------------------------------not set any transfer limit");
				T::Currency::transfer(&who, &to, value, ExistenceRequirement::AllowDeath)?;
				Self::deposit_event(Event::TransferSuccess(who.clone(), to.clone(), value));
				log::info!("-------------------------------transfer successfully");
			} else {
				let mut satisfy_times_limit = false;
				let mut satisfy_amount_limit = false;

				for i in 0..transfer_limit_id {
					if TransferLimitOwner::<T>::contains_key(&who, &i) == true {
						match MapTransferLimit::<T>::get(&i) {
							Some((block_number, TransferLimit::TimesLimit(_start_time, times))) => {
								let now = frame_system::Pallet::<T>::block_number();

								if now - 100u32.into() < block_number {
									if times <= 0 {
										Self::freeze_account(who.clone(), risk_management_id);

										Self::set_notify_method(
											who.clone(),
											next_notify_account_index,
										);

										ensure!(times > 0, Error::<T>::TransferTimesTooMany);
									} else {
										satisfy_times_limit = true;
									}
								} else {
									satisfy_times_limit = true;
								}
							},
							Some((
								_block_number,
								TransferLimit::AmountLimit(_start_time, amount),
							)) => {
								if value > amount {
									Self::freeze_account(who.clone(), risk_management_id);

									Self::set_notify_method(who.clone(), next_notify_account_index);

									ensure!(amount > value, Error::<T>::TransferValueTooLarge);
								} else {
									satisfy_amount_limit = true;
								}
							},
							_ => {
								log::info!(
									"--------------------------------not set any transfer limit"
								);
							},
						}
					}
				}
				if satisfy_amount_limit == true || satisfy_times_limit == true {
					if let Some(val) = BlockAccount::<T>::get(who.clone()) {
						if val == true {
							ensure!(!val, Error::<T>::AccountHasBeenFrozenForever);
						}
					}
					if let Some(val) = BlockTime::<T>::get(who.clone()) {
						if val == true {
							ensure!(!val, Error::<T>::AccountHasBeenFrozenTemporary);
						}
					}

					T::Currency::transfer(&who, &to, value, ExistenceRequirement::AllowDeath)?;
					Self::deposit_event(Event::TransferSuccess(who.clone(), to.clone(), value));
					log::info!("-------------------------------transfer successfully");
				}
			}

			Ok(())
		}

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

			log::info!("-------- results");

			for (acc, res) in &results {
				match res {
					Ok(()) => log::info!("[{:?}] Submitted change info", acc.id),
					Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
				}
			}

			Ok(())
		}

		fn freeze_account(who: T::AccountId, risk_management_id: u64) {
			let mut _account_freeze_status = false;

			for j in 0..risk_management_id {
				if RiskManagementOwner::<T>::contains_key(&who, &j) {
					if let Some((_, RiskManagement::AccountFreeze(freeze))) =
						MapRiskManagement::<T>::get(&j)
					{
						if freeze == true {
							BlockAccount::<T>::insert(who.clone(), true);
							_account_freeze_status = true;
							log::info!("--------------------------------freeze account forever");

							break;
						}
					}
					if _account_freeze_status == false {
						for i in 0..risk_management_id {
							if let Some((
								_block_number,
								RiskManagement::TimeFreeze(_, _freeze_time),
							)) = MapRiskManagement::<T>::get(&i)
							{
								BlockTime::<T>::insert(who.clone(), true);

								log::info!(
									"--------------------------------freeze account temporary"
								);
								break;
							}
						}
					}
				}
			}
		}

		fn set_notify_method(who: T::AccountId, next_notify_account_index: u64) {
			match T::Notification::get_mail_config_action(who.clone()) {
				Some(Action::MailWithToken(..)) => {
					log::info!("-------------------------------- mail with token status is none,set mail status true");
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
					log::info!("-------------------------------- discord with token status is none,set mail status true");
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
		fn check_account_status(who: T::AccountId, risk_management_id: u64) -> DispatchResult {
			if let Some(val) = BlockAccount::<T>::get(who.clone()) {
				if val == true {
					log::info!("-------------------------------- account has been freezed forever");
					ensure!(!val, Error::<T>::AccountHasBeenFrozenForever);
				}
			}

			if let Some(val) = BlockTime::<T>::get(who.clone()) {
				if val == true {
					for i in 0..risk_management_id {
						if let Some((
							block_number,
							RiskManagement::TimeFreeze(_start_time, freeze_time),
						)) = MapRiskManagement::<T>::get(&i)
						{
							let now = frame_system::Pallet::<T>::block_number();

							log::info!("----------- now {:?}", now);
							log::info!("----------- freeze blocks {:?}", freeze_time / 6);

							if now.saturated_into::<u64>() - freeze_time / 6
								>= block_number.saturated_into::<u64>()
							{
								log::info!("----------- freeze blocks {:?}", freeze_time % 6);
								log::info!("----------- start blocks {:?}", block_number);

								BlockAccount::<T>::mutate(who.clone(), |v| {
									*v = Some(false);
								});

								log::info!("--------------------------------unfreeze account")
							} else {
								ensure!(!val, Error::<T>::AccountHasBeenFrozenTemporary);
							}
						}
					}
				}
			}

			Ok(())
		}
	}
}
