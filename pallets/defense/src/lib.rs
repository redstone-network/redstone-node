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
	traits::{ConstU32, WrapperKeepOpaque},
	weights::{GetDispatchInfo, PostDispatchInfo, Weight},
	BoundedVec,
};
use scale_info::TypeInfo;
use sp_runtime::{
	offchain::{http, Duration},
	traits::Dispatchable,
	RuntimeDebug,
};
use sp_std::cmp::{Eq, PartialEq};

use frame_system::offchain::{AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer};
use sp_core::crypto::KeyTypeId;

type OpaqueCall<T> = WrapperKeepOpaque<<T as Config>::Call>;

type CallHash = [u8; 32];

enum CallOrHash<T: Config> {
	Call(OpaqueCall<T>, bool),
	Hash([u8; 32]),
}

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
	Mail(
		BoundedVec<u8, ConstU32<256>>, // receiver
		BoundedVec<u8, ConstU32<256>>, // title
		BoundedVec<u8, ConstU32<256>>, // message body
	),
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
	use data_encoding::BASE64;
	use frame_support::inherent::Vec;
	use primitives::permission_capture::PermissionCaptureInterface;
	use sp_io::hashing::blake2_256;

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
	pub type BlockAccount<T> = StorageValue<_, bool>;

	#[pallet::storage]
	#[pallet::getter(fn block_time)]
	pub type BlockTime<T> = StorageValue<_, bool>;

	#[pallet::storage]
	#[pallet::getter(fn mail_status)]
	pub type MailStatus<T> = StorageValue<_, bool>;

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
		PermissionTakenAccountHasPaddingCall,
		PermissionTakenAccountCallMustBeApproved,
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

			log::info!("--------------transfer_limit_id: {}", transfer_limit_id);

			if transfer_limit_id == 0 {
				match transfer_limit {
					TransferLimit::AmountLimit(_start_time, _amount) => {
						MapTransferLimit::<T>::insert(
							transfer_limit_id,
							(frame_system::Pallet::<T>::block_number(), transfer_limit.clone()),
						);
						TransferLimitOwner::<T>::insert(&who, transfer_limit_id, ());
						NextTransferLimitId::<T>::put(transfer_limit_id.saturating_add(One::one()));
						log::info!("--------------------------------set transfer amount");

						Self::deposit_event(Event::TransferAmountLimitSet(
							who.clone(),
							transfer_limit,
						));
					},
					TransferLimit::TimesLimit(_start_time, _times) => {
						MapTransferLimit::<T>::insert(
							transfer_limit_id,
							(frame_system::Pallet::<T>::block_number(), transfer_limit.clone()),
						);
						TransferLimitOwner::<T>::insert(&who, transfer_limit_id, ());
						NextTransferLimitId::<T>::put(transfer_limit_id.saturating_add(One::one()));

						log::info!("--------------------------------set transfer times");
						Self::deposit_event(Event::TransferTimesLimitSet(
							who.clone(),
							transfer_limit,
						));
					},
				}
			} else {
				match transfer_limit {
					TransferLimit::AmountLimit(_, _) => {
						let mut transfer_amount_limit_set = false;

						for i in 0..transfer_limit_id {
							log::info!("{:?}", MapTransferLimit::<T>::get(i));
							if let Some((_, TransferLimit::AmountLimit(_set_time, _set_amount))) =
								MapTransferLimit::<T>::get(i)
							{
								log::info!(
									"--------------------------------update transfer amount"
								);
								MapTransferLimit::<T>::mutate(&i, |v| {
									*v = Some((
										frame_system::Pallet::<T>::block_number(),
										transfer_limit.clone(),
									))
								});
								log::info!("change transfer amount limit successfully");
								transfer_amount_limit_set = true;
								Self::deposit_event(Event::TransferAmountLimitUpdated(
									who.clone(),
									transfer_limit,
								));
								break
							}
						}

						if transfer_amount_limit_set == false {
							log::info!("--------------------------------set transfer amount");
							MapTransferLimit::<T>::insert(
								transfer_limit_id,
								(frame_system::Pallet::<T>::block_number(), transfer_limit.clone()),
							);
							TransferLimitOwner::<T>::insert(&who, transfer_limit_id, ());
							NextTransferLimitId::<T>::put(
								transfer_limit_id.saturating_add(One::one()),
							);

							Self::deposit_event(Event::TransferAmountLimitSet(
								who.clone(),
								transfer_limit,
							));
						}
					},

					TransferLimit::TimesLimit(_, _) => {
						let mut transfer_times_limit_set = false;

						for i in 0..transfer_limit_id {
							log::info!("{:?}", MapTransferLimit::<T>::get(i));
							if let Some((_, TransferLimit::TimesLimit(_set_time, _set_times))) =
								MapTransferLimit::<T>::get(i)
							{
								log::info!("--------------------------------update transfer times");
								MapTransferLimit::<T>::mutate(&i, |v| {
									*v = Some((
										frame_system::Pallet::<T>::block_number(),
										transfer_limit.clone(),
									))
								});
								log::info!("change transfer times limit successfully");
								transfer_times_limit_set = true;
								Self::deposit_event(Event::TransferTimesLimitUpdated(
									who.clone(),
									transfer_limit,
								));
								break
							}
						}

						if transfer_times_limit_set == false {
							log::info!("--------------------------------set transfer times");
							MapTransferLimit::<T>::insert(
								transfer_limit_id,
								(frame_system::Pallet::<T>::block_number(), transfer_limit.clone()),
							);
							TransferLimitOwner::<T>::insert(&who, transfer_limit_id, ());
							NextTransferLimitId::<T>::put(
								transfer_limit_id.saturating_add(One::one()),
							);

							Self::deposit_event(Event::TransferTimesLimitSet(
								who.clone(),
								transfer_limit,
							));
						}
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

			log::info!("--------------risk_management_id: {}", risk_management_id);

			if risk_management_id == 0 {
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
					RiskManagement::Mail(_, _, _) => {
						log::info!("--------------------------------set notify mail");
						Self::deposit_event(Event::RiskManagementMailSet(
							who.clone(),
							risk_management.clone(),
						));
					},
				}
			} else {
				let mut freeze_time_set = false;
				let mut freeze_account_set = false;
				let mut notify_mail_set = false;

				match risk_management {
					RiskManagement::TimeFreeze(_, _) => {
						for i in 0..risk_management_id {
							if let Some((_, RiskManagement::TimeFreeze(_, _))) =
								MapRiskManagement::<T>::get(&i)
							{
								freeze_time_set = true;
								ensure!(
									!MapRiskManagement::<T>::contains_key(&i),
									Error::<T>::FreezeTimeHasSet
								);
							}
						}
						if freeze_time_set == false {
							MapRiskManagement::<T>::insert(
								risk_management_id,
								(
									frame_system::Pallet::<T>::block_number(),
									risk_management.clone(),
								),
							);
							RiskManagementOwner::<T>::insert(&who, risk_management_id, ());
							NextRiskManagementId::<T>::put(
								risk_management_id.saturating_add(One::one()),
							);

							Self::deposit_event(Event::RiskManagementTimeFreezeSet(
								who.clone(),
								risk_management.clone(),
							));
						}
					},
					RiskManagement::AccountFreeze(_) => {
						for i in 0..risk_management_id {
							if let Some((_, RiskManagement::AccountFreeze(_))) =
								MapRiskManagement::<T>::get(&i)
							{
								freeze_account_set = true;
								ensure!(
									!MapRiskManagement::<T>::contains_key(&i),
									Error::<T>::FreezeAccountHasSet
								);
							}
						}
						if freeze_account_set == false {
							MapRiskManagement::<T>::insert(
								risk_management_id,
								(
									frame_system::Pallet::<T>::block_number(),
									risk_management.clone(),
								),
							);
							RiskManagementOwner::<T>::insert(&who, risk_management_id, ());
							NextRiskManagementId::<T>::put(
								risk_management_id.saturating_add(One::one()),
							);

							Self::deposit_event(Event::RiskManagementAccountFreezeSet(
								who.clone(),
								risk_management.clone(),
							));
						}
					},
					RiskManagement::Mail(_, _, _) => {
						for i in 0..risk_management_id {
							if let Some((_, RiskManagement::Mail(_, _, _))) =
								MapRiskManagement::<T>::get(&i)
							{
								notify_mail_set = true;

								log::info!("--------------------------------update notify email");

								// MapTransferLimit::<T>::insert(i, transfer_limit.clone());
								MapRiskManagement::<T>::mutate(&i, |v| {
									*v = Some((
										frame_system::Pallet::<T>::block_number(),
										risk_management.clone(),
									))
								});
								log::info!("change notify mail successfully");

								Self::deposit_event(Event::RiskManagementMailUpdated(
									who.clone(),
									risk_management.clone(),
								));
							}
						}
						if notify_mail_set == false {
							log::info!("--------------------------------set notify mail");
							MapRiskManagement::<T>::insert(
								risk_management_id,
								(
									frame_system::Pallet::<T>::block_number(),
									risk_management.clone(),
								),
							);
							RiskManagementOwner::<T>::insert(&who, risk_management_id, ());
							NextRiskManagementId::<T>::put(
								risk_management_id.saturating_add(One::one()),
							);
							Self::deposit_event(Event::RiskManagementMailSet(
								who.clone(),
								risk_management.clone(),
							));
						}
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

			if T::PermissionCaptureInterface::is_account_permission_taken(who.clone()) {
				let call = Call::<T>::safe_transfer { to: to.clone(), value };
				let data = call.encode();
				let call_wrapper = OpaqueCall::<T>::from_encoded(data.clone());
				let call_hash = blake2_256(call_wrapper.encoded());

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
					return Ok(())
				}
			}

			if transfer_limit_id == 0 {
				log::info!("--------------------------------not set any transfer limit");
				T::Currency::transfer(&who, &to, value, ExistenceRequirement::AllowDeath)?;
				Self::deposit_event(Event::TransferSuccess(who.clone(), to.clone(), value));
				log::info!("-------------------------------transfer successfully");
			} else {
				let mut satisfy_times_limit = false;
				let mut satisfy_amount_limit = false;

				let mut set_freeze_account = false;

				for i in 0..transfer_limit_id {
					match MapTransferLimit::<T>::get(&i) {
						Some((block_number, TransferLimit::TimesLimit(_start_time, times))) => {
							let now = frame_system::Pallet::<T>::block_number();
							if now - 100u32.into() < block_number {
								if times <= 0 {
									for i in 0..risk_management_id {
										if let Some((_, RiskManagement::AccountFreeze(freeze))) =
											MapRiskManagement::<T>::get(&i)
										{
											if freeze == true {
												BlockAccount::<T>::put(true);
												set_freeze_account = true;
												log::info!(
													"--------------------------------freeze account forever"
												);
												break
											}
										}
									}
									if set_freeze_account == false {
										for i in 0..risk_management_id {
											if let Some((
												_block_number,
												RiskManagement::TimeFreeze(_, _freeze_time),
											)) = MapRiskManagement::<T>::get(&i)
											{
												BlockTime::<T>::put(true);
												for i in 0..risk_management_id {
													if let Some((
														_,
														RiskManagement::Mail(_, _, _),
													)) = MapRiskManagement::<T>::get(&i)
													{
														log::info!("--------------------------------set mail status true");
														MailStatus::<T>::put(true);
														break
													}
												}
												log::info!(
													"--------------------------------freeze account temporary "
												);
												break
											}
										}
									}
									for i in 0..risk_management_id {
										if let Some((_, RiskManagement::Mail(_, _, _))) =
											MapRiskManagement::<T>::get(&i)
										{
											match MailStatus::<T>::get() {
												Some(val) =>
													if val != true {
														log::info!("-------------------------------- mail status true is false,set mail status true");
														MailStatus::<T>::put(true);
													},
												None => {
													log::info!("-------------------------------- mail status true is none,set mail status true");
													MailStatus::<T>::put(true);
												},
											}
											break
										}
									}
									ensure!(times > 0, Error::<T>::TransferTimesTooMany);
								} else {
									satisfy_times_limit = true;
								}
							} else {
								satisfy_times_limit = true;
							}
						},
						Some((_block_number, TransferLimit::AmountLimit(_start_time, amount))) => {
							if value > amount {
								for i in 0..risk_management_id {
									if let Some((_, RiskManagement::AccountFreeze(freeze))) =
										MapRiskManagement::<T>::get(&i)
									{
										if freeze == true {
											BlockAccount::<T>::put(true);
											set_freeze_account = true;

											log::info!(
												"--------------------------------freeze account forever"
											);
											break
										}
									}
								}

								for i in 0..risk_management_id {
									if let Some((_, RiskManagement::Mail(_, _, _))) =
										MapRiskManagement::<T>::get(&i)
									{
										match MailStatus::<T>::get() {
											Some(val) =>
												if val != true {
													log::info!("--------------------------------mail status true is false,set mail status true");

													MailStatus::<T>::put(true);
												},
											None => {
												log::info!("-------------------------------- mail status true is none,set mail status true");
												MailStatus::<T>::put(true);
											},
										}
										break
									}
								}

								if set_freeze_account == false {
									for i in 0..risk_management_id {
										if let Some((
											_block_number,
											RiskManagement::TimeFreeze(_, _freeze_time),
										)) = MapRiskManagement::<T>::get(&i)
										{
											BlockTime::<T>::put(true);

											log::info!(
												"--------------------------------freeze account temporary "
											);
											break
										}
									}
								}
								ensure!(amount > value, Error::<T>::TransferValueTooLarge);
							}
							satisfy_amount_limit = true;
						},
						_ => {
							log::info!(
								"--------------------------------not set any transfer limit"
							);
						},
					}
				}
				if satisfy_amount_limit == true || satisfy_times_limit == true {
					match BlockAccount::<T>::get() {
						Some(val) => {
							log::info!(
								"-------------------------------- account has been freezed forever"
							);
							ensure!(!val, Error::<T>::AccountHasBeenFrozenForever);
						},
						None => {
							log::info!("--------------------------------health account");
						},
					}
					match BlockTime::<T>::get() {
						Some(val) =>
							if val == true {
								for i in 0..risk_management_id {
									if let Some((
										block_number,
										RiskManagement::TimeFreeze(_start_time, freeze_time),
									)) = MapRiskManagement::<T>::get(&i)
									{
										let now = frame_system::Pallet::<T>::block_number();

										if now.saturated_into::<u64>() - freeze_time % 6 >=
											block_number.saturated_into::<u64>()
										{
											BlockAccount::<T>::put(false);

											if let Some(val) = MailStatus::<T>::get() {
												if val == true {
													MailStatus::<T>::put(false);
													log::info!(
														"-------------------------------- reactivate email notification"
													);
												}
											}

											log::info!(
												"--------------------------------unfreeze account"
											)
										} else {
											ensure!(
												!val,
												Error::<T>::AccountHasBeenFrozenTemporary
											);
										}
									}
								}
							} else {
								log::info!("--------------------------------health amount")
							},
						None => {
							log::info!("--------------------------------health amount")
						},
					}

					T::Currency::transfer(&who, &to, value, ExistenceRequirement::AllowDeath)?;
					Self::deposit_event(Event::TransferSuccess(who.clone(), to.clone(), value));
					log::info!("-------------------------------transfer successfully");
				} else {
					log::info!(
						"-------------------------------condition not satisfy,transfer failed!!!"
					);
				}
			}

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn reset_email_status(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;

			if let Some(val) = MailStatus::<T>::get() {
				if val == true {
					MailStatus::<T>::put(false);
					log::info!("-------------------------------- reactivate email notification");
				}
			}

			Ok(().into())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("Hi World from defense-pallet workers!: {:?}", block_number);
			let risk_management_id = NextRiskManagementId::<T>::get().unwrap_or_default();

			// 检查是否需要发送邮件
			match MailStatus::<T>::get() {
				Some(val) => {
					if val == true {
						for i in 0..risk_management_id {
							if let Some((_, RiskManagement::Mail(receiver, title, message_body))) =
								MapRiskManagement::<T>::get(&i)
							{
								log::info!("--------------------------------send email");

								let to_email = match scale_info::prelude::string::String::from_utf8(
									receiver.to_vec(),
								) {
									Ok(v) => v,
									Err(e) => {
										log::info!("------decode receiver error  {:?}", e);
										continue
									},
								};

								let subject = match scale_info::prelude::string::String::from_utf8(
									title.to_vec(),
								) {
									Ok(v) => v,
									Err(e) => {
										log::info!("------decode subject error  {:?}", e);
										continue
									},
								};

								let content = match scale_info::prelude::string::String::from_utf8(
									message_body.to_vec(),
								) {
									Ok(v) => v,
									Err(e) => {
										log::info!("------decode content error  {:?}", e);
										continue
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

								match Self::send_email_info(url) {
									Ok(val) => {
										log::info!("email send successfully {:?}", val);
										match Self::send_signed_tx() {
											Ok(_) => log::info!("reset email status as false"),
											Err(e) => {
												log::info!(
													"reset email status as false failed {:?}",
													e
												);
											},
										};
									},
									Err(e) => log::info!("email send failed {:?}", e),
								};
							}
						}
					} else {
						// log::info!("--------------------------------no need to send email");
					}
				},
				None => {
					// log::info!("--------------------------------no find email info");
				},
			}
		}
	}
	impl<T: Config> Pallet<T> {
		fn send_email_info(url: &str) -> Result<u64, http::Error> {
			// prepare for send request
			// log::info!("--------------request url {:?}", url);
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
				log::info!("email send successfully")
			}
			let body = response.body().collect::<Vec<u8>>();
			let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
				log::warn!("No UTF8 body");
				http::Error::Unknown
			})?;

			log::info!("get return value: {}", body_str);

			Ok(0)
		}
		fn send_signed_tx() -> Result<(), &'static str> {
			let signer = Signer::<T, T::AuthorityId>::all_accounts();
			if !signer.can_sign() {
				return Err(
					"No local accounts available. Consider adding one via `author_insertKey` RPC.",
				)
			}

			let results = signer.send_signed_transaction(|_account| Call::reset_email_status {});

			log::info!("-------- results");

			for (acc, res) in &results {
				match res {
					Ok(()) => log::info!("[{:?}] Submitted change info", acc.id),
					Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
				}
			}

			Ok(())
		}
	}
}
