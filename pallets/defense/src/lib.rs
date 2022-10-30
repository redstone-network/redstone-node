#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::ConstU32, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::cmp::{Eq, PartialEq};

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
	use frame_support::pallet_prelude::*;
	use frame_support::traits::ReservableCurrency;
	use frame_support::traits::{Currency, ExistenceRequirement};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::One;
	use sp_runtime::traits::SaturatedConversion;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
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
								break;
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
								break;
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

			if transfer_limit_id == 0 {
				log::info!("--------------------------------not set any transfer limit");
				T::Currency::transfer(&who, &to, value, ExistenceRequirement::AllowDeath)?;
				Self::deposit_event(Event::TransferSuccess(who.clone(), to.clone(), value));
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
												log::info!(
													"--------------------------------freeze account forever"
												);
												BlockAccount::<T>::put(true);
												set_freeze_account = true;
												break;
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
												log::info!(
													"--------------------------------freeze account for some time "
												);
												BlockTime::<T>::put(true);
												break;
											}
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
											log::info!(
												"--------------------------------freeze account forever"
											);
											BlockAccount::<T>::put(true);
											set_freeze_account = true;
											break;
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
											log::info!(
												"--------------------------------freeze account for some time "
											);
											BlockTime::<T>::put(true);
											break;
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
				if satisfy_amount_limit == true && satisfy_times_limit == true {
					match BlockAccount::<T>::get() {
						Some(val) => {
							ensure!(!val, Error::<T>::AccountHasBeenFrozenForever);
						},
						None => {
							log::info!("--------------------------------health account")
						},
					}

					match BlockTime::<T>::get() {
						Some(val) => {
							if val == true {
								for i in 0..risk_management_id {
									if let Some((
										block_number,
										RiskManagement::TimeFreeze(_start_time, freeze_time),
									)) = MapRiskManagement::<T>::get(&i)
									{
										let now = frame_system::Pallet::<T>::block_number();

										if now.saturated_into::<u64>() - freeze_time % 6
											>= block_number.saturated_into::<u64>()
										{
											BlockAccount::<T>::put(false);
											log::info!(
												"--------------------------------unfreeze amount"
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
							}
						},
						None => {
							log::info!("--------------------------------health amount")
						},
					}

					T::Currency::transfer(&who, &to, value, ExistenceRequirement::AllowDeath)?;
					Self::deposit_event(Event::TransferSuccess(who.clone(), to.clone(), value));
				}
			}

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("Hi World from defense-pallet workers!: {:?}", block_number);
		}
	}
	impl<T: Config> Pallet<T> {}
}
