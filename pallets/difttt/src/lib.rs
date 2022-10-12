#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::ConstU32, BoundedVec};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::cmp::{Eq, PartialEq};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use pallet::*;
pub use weights::WeightInfo;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
// #[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum Triger<Balance> {
	Timer(u64, u64),    //insert_time_millis_seconds,  timer_millis_seconds
	Schedule(u64, u64), //insert_time_millis_seconds,  timestamp
	PriceGT(u64, u64),  //insert_time_millis_seconds,  price   //todo,price use float
	PriceLT(u64, u64),  //insert_time_millis_seconds,  price   //todo,price use float
	TransferProtect(u64, Balance, u64), /* limit amout per transfer, transfer count limit per

						* 100 blocks */
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
//#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
//#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum Action<AccountId> {
	MailWithToken(
		BoundedVec<u8, ConstU32<128>>,
		BoundedVec<u8, ConstU32<256>>,
		BoundedVec<u8, ConstU32<128>>,
		BoundedVec<u8, ConstU32<128>>,
		BoundedVec<u8, ConstU32<256>>,
	),
	/* url, encrypted access_token
	 * by asymmetric encryption,
	 * revicer, title, body */
	BuyToken(
		AccountId,
		BoundedVec<u8, ConstU32<32>>,
		u64,
		BoundedVec<u8, ConstU32<32>>,
		BoundedVec<u8, ConstU32<128>>,
	), /* Address, SellTokenName, SellAmount, BuyTokenName, info-mail-recevicer */
	MailByLocalServer(
		BoundedVec<u8, ConstU32<128>>,
		BoundedVec<u8, ConstU32<128>>,
		BoundedVec<u8, ConstU32<256>>,
	), //revicer, title, body
	Slack(BoundedVec<u8, ConstU32<256>>, BoundedVec<u8, ConstU32<256>>), /*slack_hook_url,
	                                                                      * message */
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
// #[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Recipe {
	triger_id: u64,
	action_id: u64,
	enable: bool,
	is_forever: bool,
	times: u64,
	max_times: u64,
	done: bool,
	last_triger_timestamp: u64,
}

#[frame_support::pallet]
pub mod pallet {
	pub use crate::weights::WeightInfo;
	use crate::{Action, Recipe, Triger};
	use codec::alloc::string::ToString;
	use data_encoding::BASE64;
	use frame_support::{ensure, pallet_prelude::*, traits::UnixTime};
	use frame_system::{
		offchain::{AppCrypto, CreateSignedTransaction, SignedPayload, Signer, SigningTypes},
		pallet_prelude::*,
	};
	use orml_traits::{MultiCurrency, MultiReservableCurrency, TransferProtectInterface};
	use sp_core::{crypto::KeyTypeId, offchain::Timestamp};
	use sp_runtime::{
		offchain::{
			http,
			storage::StorageValueRef,
			storage_lock::{BlockAndTime, StorageLock},
			Duration,
		},
		traits::{BlockNumberProvider, One},
	};

	use sp_std::{collections::btree_map::BTreeMap, prelude::*, str};

	use frame_system::offchain::SendUnsignedTransaction;
	pub use primitives::{
		currency::{
			TokenInfo, ACA, AUSD, BNC, DOT, KAR, KBTC, KINT, KSM, KUSD, LCDOT, LDOT, LKSM, PHA,
			RENBTC, VSKSM,
		},
		TokenSymbol, TradingPair,
	};

	use sp_runtime::DispatchResult;

	pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"dift");
	const FETCH_TIMEOUT_PERIOD: u64 = 3000; // in milli-seconds
	const LOCK_TIMEOUT_EXPIRATION: u64 = FETCH_TIMEOUT_PERIOD + 1000; // in milli-seconds
	const LOCK_BLOCK_EXPIRATION: u32 = 3; // in block number

	pub mod crypto {
		use crate::KEY_TYPE;
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
		impl
			frame_system::offchain::AppCrypto<
				<Sr25519Signature as Verify>::Signer,
				Sr25519Signature,
			> for TestAuthId
		{
			type RuntimeAppPublic = Public;
			type GenericSignature = sp_core::sr25519::Signature;
			type GenericPublic = sp_core::sr25519::Public;
		}
	}

	type BalanceOf<T> =
		<<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type TimeProvider: UnixTime;

		/// The overarching dispatch call type.
		type Call: From<Call<Self>>;

		/// A configuration for base priority of unsigned transactions.
		///
		/// This is exposed so that it can be tuned for particular runtime, when
		/// multiple pallets send unsigned transactions.
		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		type Currency: MultiReservableCurrency<Self::AccountId>;

		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage

	#[pallet::storage]
	#[pallet::getter(fn triger_owner)]
	pub type TrigerOwner<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u64, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn action_owner)]
	pub type ActionOwner<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u64, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn recipe_owner)]
	pub type RecipeOwner<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u64, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn map_triger)]
	pub(super) type MapTriger<T: Config> = StorageMap<_, Twox64Concat, u64, Triger<BalanceOf<T>>>;

	#[pallet::storage]
	#[pallet::getter(fn map_action)]
	pub(super) type MapAction<T: Config> = StorageMap<_, Twox64Concat, u64, Action<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn map_recipe)]
	pub(super) type MapRecipe<T: Config> = StorageMap<_, Twox64Concat, u64, Recipe>;

	#[pallet::storage]
	#[pallet::getter(fn next_triger_id)]
	pub type NextTrigerId<T: Config> = StorageValue<_, u64>;
	#[pallet::storage]
	#[pallet::getter(fn next_action_id)]
	pub type NextActionId<T: Config> = StorageValue<_, u64>;
	#[pallet::storage]
	#[pallet::getter(fn next_recipe_id)]
	pub type NextRecipeId<T: Config> = StorageValue<_, u64>;

	#[pallet::storage]
	#[pallet::getter(fn amount_limit)]
	pub type AmountLimit<T: Config> = StorageValue<_, BalanceOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn tx_block_limit)]
	pub type TxBlockLimit<T: Config> = StorageValue<_, u64>;

	/// Payload used by set recipe done to submit a transaction.
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct RecipeDonePayload<Public, BlockNumber> {
		block_number: BlockNumber,
		recipe_id: u64,
		public: Public,
	}

	impl<T: SigningTypes> SignedPayload<T> for RecipeDonePayload<T::Public, T::BlockNumber> {
		fn public(&self) -> T::Public {
			self.public.clone()
		}
	}

	/// Payload used by update recipe times to submit a transaction.
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct RecipeTimesPayload<Public, BlockNumber> {
		block_number: BlockNumber,
		recipe_id: u64,
		times: u64,
		timestamp: u64,
		public: Public,
	}

	impl<T: SigningTypes> SignedPayload<T> for RecipeTimesPayload<T::Public, T::BlockNumber> {
		fn public(&self) -> T::Public {
			self.public.clone()
		}
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		TrigerCreated(u64, Triger<BalanceOf<T>>),
		ActionCreated(u64, Action<T::AccountId>),
		RecipeCreated(u64, Recipe),
		RecipeRemoved(u64),
		RecipeTurnOned(u64),
		RecipeTurnOffed(u64),
		RecipeDone(u64),
		RecipeTrigerTimesUpdated(u64, u64, u64),
		TokenBought(T::AccountId, Vec<u8>, u64, Vec<u8>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		/// Errors should have helpful documentation associated with them.
		TrigerIdNotExist,
		ActionIdNotExist,
		RecipeIdNotExist,
		NotOwner,
		OffchainUnsignedTxError,
		InsufficientBalance,
		NoLocalAccounts,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// create_trigerid
		#[pallet::weight(<T as Config>::WeightInfo::create_triger())]
		pub fn create_triger(origin: OriginFor<T>, triger: Triger<BalanceOf<T>>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			let triger_id = NextTrigerId::<T>::get().unwrap_or_default();

			MapTriger::<T>::insert(triger_id, triger.clone());
			TrigerOwner::<T>::insert(user, triger_id, ());
			NextTrigerId::<T>::put(triger_id.saturating_add(One::one()));

			match triger.clone() {
				Triger::TransferProtect(_, amout_limit, blocks_amount) => {
					AmountLimit::<T>::put(amout_limit);
					TxBlockLimit::<T>::put(blocks_amount);
				},
				_ => {},
			}
			Self::deposit_event(Event::TrigerCreated(triger_id, triger));

			Ok(())
		}

		/// create_action
		#[pallet::weight(<T as Config>::WeightInfo::create_action())]
		pub fn create_action(origin: OriginFor<T>, action: Action<T::AccountId>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			let action_id = NextActionId::<T>::get().unwrap_or_default();

			MapAction::<T>::insert(action_id, action.clone());
			ActionOwner::<T>::insert(user, action_id, ());
			NextActionId::<T>::put(action_id.saturating_add(One::one()));

			Self::deposit_event(Event::ActionCreated(action_id, action));

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::create_recipe())]
		pub fn create_recipe(
			origin: OriginFor<T>,
			triger_id: u64,
			action_id: u64,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;
			let recipe_id = NextRecipeId::<T>::get().unwrap_or_default();

			ensure!(MapTriger::<T>::contains_key(&triger_id), Error::<T>::TrigerIdNotExist);
			ensure!(MapAction::<T>::contains_key(&action_id), Error::<T>::ActionIdNotExist);

			let triger = MapTriger::<T>::get(triger_id);
			let recipe = Recipe {
				triger_id,
				action_id,
				enable: true,
				is_forever: match triger {
					Some(Triger::Timer(_, _)) => true,
					_ => false,
				},
				times: 0,
				max_times: 1,
				done: false,
				last_triger_timestamp: 0,
			};

			MapRecipe::<T>::insert(recipe_id, recipe.clone());
			RecipeOwner::<T>::insert(user, recipe_id, ());
			NextRecipeId::<T>::put(recipe_id.saturating_add(One::one()));

			Self::deposit_event(Event::RecipeCreated(recipe_id, recipe));

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::del_recipe())]
		pub fn del_recipe(origin: OriginFor<T>, recipe_id: u64) -> DispatchResult {
			let user = ensure_signed(origin)?;

			ensure!(MapRecipe::<T>::contains_key(&recipe_id), Error::<T>::RecipeIdNotExist);
			ensure!(RecipeOwner::<T>::contains_key(&user, &recipe_id), Error::<T>::NotOwner);

			RecipeOwner::<T>::remove(user, recipe_id);
			MapRecipe::<T>::remove(recipe_id);

			Self::deposit_event(Event::RecipeRemoved(recipe_id));

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::turn_on_recipe())]
		pub fn turn_on_recipe(origin: OriginFor<T>, recipe_id: u64) -> DispatchResult {
			let user = ensure_signed(origin)?;

			ensure!(MapRecipe::<T>::contains_key(&recipe_id), Error::<T>::RecipeIdNotExist);
			ensure!(RecipeOwner::<T>::contains_key(&user, &recipe_id), Error::<T>::NotOwner);

			MapRecipe::<T>::try_mutate(recipe_id, |recipe| -> DispatchResult {
				if let Some(recipe) = recipe {
					recipe.enable = true;
					Self::deposit_event(Event::RecipeTurnOned(recipe_id));
				}
				Ok(())
			})?;

			Ok(())
		}

		#[pallet::weight(<T as Config>::WeightInfo::turn_off_recipe())]
		pub fn turn_off_recipe(origin: OriginFor<T>, recipe_id: u64) -> DispatchResult {
			let user = ensure_signed(origin)?;

			ensure!(MapRecipe::<T>::contains_key(&recipe_id), Error::<T>::RecipeIdNotExist);
			ensure!(RecipeOwner::<T>::contains_key(user, &recipe_id), Error::<T>::NotOwner);

			MapRecipe::<T>::try_mutate(recipe_id, |recipe| -> DispatchResult {
				if let Some(recipe) = recipe {
					recipe.enable = false;
					Self::deposit_event(Event::RecipeTurnOffed(recipe_id));
				}
				Ok(())
			})?;

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn submit_recipe_done_with_signed_payload(
			origin: OriginFor<T>,
			recipe_done_payload: RecipeDonePayload<T::Public, T::BlockNumber>,
			_signature: T::Signature,
		) -> DispatchResult {
			// This ensures that the function can only be called via unsigned transaction.
			ensure_none(origin)?;
			let recipe_id = recipe_done_payload.recipe_id;

			ensure!(MapRecipe::<T>::contains_key(&recipe_id), Error::<T>::RecipeIdNotExist);

			MapRecipe::<T>::try_mutate(recipe_id, |recipe| -> DispatchResult {
				if let Some(recipe) = recipe {
					recipe.done = true;
					Self::deposit_event(Event::RecipeDone(recipe_id));
				}
				Ok(())
			})?;

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn submit_recipe_triger_times_with_signed_payload(
			origin: OriginFor<T>,
			recipe_times_payload: RecipeTimesPayload<T::Public, T::BlockNumber>,
			_signature: T::Signature,
		) -> DispatchResult {
			// This ensures that the function can only be called via unsigned transaction.
			ensure_none(origin)?;

			log::info!("###### in submit_recipe_triger_times_with_signed_payload.");

			ensure!(
				MapRecipe::<T>::contains_key(&recipe_times_payload.recipe_id),
				Error::<T>::RecipeIdNotExist
			);

			MapRecipe::<T>::try_mutate(
				recipe_times_payload.recipe_id,
				|recipe| -> DispatchResult {
					if let Some(recipe) = recipe {
						recipe.times = recipe_times_payload.times;
						recipe.last_triger_timestamp = recipe_times_payload.timestamp;
						Self::deposit_event(Event::RecipeTrigerTimesUpdated(
							recipe_times_payload.recipe_id,
							recipe_times_payload.times,
							recipe_times_payload.timestamp,
						));
					}
					Ok(())
				},
			)?;

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("###### Hello from pallet-difttt-offchain-worker.");

			//let timestamp_now = T::TimeProvider::now();
			let timestamp_now = sp_io::offchain::timestamp();
			log::info!("###### Current time: {:?} ", timestamp_now.unix_millis());

			let store_hashmap_recipe = StorageValueRef::persistent(b"difttt_ocw::recipe_task");

			let mut map_recipe_task: BTreeMap<u64, Recipe>;
			if let Ok(Some(info)) = store_hashmap_recipe.get::<BTreeMap<u64, Recipe>>() {
				map_recipe_task = info;
			} else {
				map_recipe_task = BTreeMap::new();
			}

			let mut lock = StorageLock::<BlockAndTime<Self>>::with_block_and_time_deadline(
				b"offchain-demo::lock",
				LOCK_BLOCK_EXPIRATION,
				Duration::from_millis(LOCK_TIMEOUT_EXPIRATION),
			);

			let mut map_running_action_recipe_task: BTreeMap<u64, Recipe> = BTreeMap::new();
			if let Ok(_guard) = lock.try_lock() {
				Self::filter_running_recipe(&mut map_recipe_task);
				Self::check_triger_recipe(
					&mut map_recipe_task,
					&mut map_running_action_recipe_task,
					timestamp_now,
				);
				store_hashmap_recipe.set(&map_recipe_task);
			};

			Self::run_action(&mut map_running_action_recipe_task, block_number, timestamp_now);
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// Firstly let's check that we call the right function.
			let valid_tx = |provide| {
				ValidTransaction::with_tag_prefix("ocw-difttt")
					.priority(T::UnsignedPriority::get())
					.and_provides([&provide])
					.longevity(3)
					.propagate(true)
					.build()
			};

			match call {
				Call::submit_recipe_done_with_signed_payload {
					recipe_done_payload: ref payload,
					ref signature,
				} => {
					let signature_valid =
						SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
					if !signature_valid {
						return InvalidTransaction::BadProof.into()
					}

					valid_tx(b"submit_recipe_done_with_signed_payload".to_vec())
				},
				Call::submit_recipe_triger_times_with_signed_payload {
					recipe_times_payload: ref payload,
					ref signature,
				} => {
					let signature_valid =
						SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
					if !signature_valid {
						return InvalidTransaction::BadProof.into()
					}

					valid_tx(b"submit_recipe_triger_times_with_signed_payload".to_vec())
				},
				_ => InvalidTransaction::Call.into(),
			}
		}
	}

	impl<T: Config> Pallet<T> {
		fn filter_running_recipe(
			map_recipe_task: &mut BTreeMap<u64, Recipe>,
		) -> Result<u64, Error<T>> {
			for (recipe_id, recipe) in MapRecipe::<T>::iter() {
				if recipe.enable && !recipe.done {
					if !map_recipe_task.contains_key(&recipe_id) {
						log::info!("###### map_recipe_task.insert {:?}", &recipe_id);
						map_recipe_task.insert(
							recipe_id,
							Recipe {
								triger_id: recipe.triger_id,
								action_id: recipe.action_id,
								enable: true,
								is_forever: recipe.is_forever,
								times: 0,
								max_times: 1,
								done: false,
								last_triger_timestamp: 0,
							},
						);
					}
				} else {
					log::info!("###### map_recipe_task.remove {:?}", &recipe_id);
					map_recipe_task.remove(&recipe_id);
				};
			}

			Ok(0)
		}

		fn check_triger_recipe(
			map_recipe_task: &mut BTreeMap<u64, Recipe>,
			map_running_action_recipe_task: &mut BTreeMap<u64, Recipe>,
			timestamp_now: Timestamp,
		) -> Result<u64, Error<T>> {
			for (recipe_id, recipe) in map_recipe_task.iter_mut() {
				let triger = MapTriger::<T>::get(recipe.triger_id);

				match triger {
					Some(Triger::Timer(insert_time, timer_millis_seconds)) => {
						if insert_time + recipe.times * timer_millis_seconds <
							timestamp_now.unix_millis()
						{
							(*recipe).times += 1;
							log::info!(
								"###### recipe {:?} Current Triger times: {:?} ",
								recipe_id,
								recipe.times
							);

							map_running_action_recipe_task.insert(*recipe_id, recipe.clone());
						}
					},
					Some(Triger::Schedule(_, timestamp)) => {
						if timestamp < timestamp_now.unix_millis() {
							(*recipe).times += 1;
							(*recipe).done = true;

							map_running_action_recipe_task.insert(*recipe_id, recipe.clone());
						}
					},
					_ => {},
				}
			}

			Ok(0)
		}

		fn run_action(
			map_running_action_recipe_task: &mut BTreeMap<u64, Recipe>,
			block_number: T::BlockNumber,
			timestamp_now: Timestamp,
		) -> Result<u64, Error<T>> {
			for (recipe_id, recipe) in map_running_action_recipe_task.iter() {
				let action = MapAction::<T>::get(recipe.action_id);
				match action {
					Some(Action::MailWithToken(url, token, revicer, title, body)) => {
						let url = match scale_info::prelude::string::String::from_utf8(url.to_vec())
						{
							Ok(v) => v,
							Err(e) => {
								log::info!("###### decode url error  {:?}", e);
								continue
							},
						};

						let token =
							match scale_info::prelude::string::String::from_utf8(token.to_vec()) {
								Ok(v) => v,
								Err(e) => {
									log::info!("###### decode token error  {:?}", e);
									continue
								},
							};

						let revicer = match scale_info::prelude::string::String::from_utf8(
							revicer.to_vec(),
						) {
							Ok(v) => v,
							Err(e) => {
								log::info!("###### decode revicer error  {:?}", e);
								continue
							},
						};

						let title =
							match scale_info::prelude::string::String::from_utf8(title.to_vec()) {
								Ok(v) => v,
								Err(e) => {
									log::info!("###### decode title error  {:?}", e);
									continue
								},
							};

						let body =
							match scale_info::prelude::string::String::from_utf8(body.to_vec()) {
								Ok(v) => v,
								Err(e) => {
									log::info!("###### decode body error  {:?}", e);
									continue
								},
							};

						let options = scale_info::prelude::format!(
							"/email --url={} --token={} --revicer={} --title={} --body={}",
							url,
							token,
							revicer,
							title,
							body,
						);

						log::info!("###### publish_task mail options  {:?}", options);

						let _rt = match Self::publish_task(
							"registry.cn-shenzhen.aliyuncs.com/difttt/email:latest",
							&options,
							3,
						) {
							Ok(_i) => {
								log::info!("###### publish_task mail ok");
								Self::update_recipe_times(
									block_number,
									*recipe_id,
									recipe.times,
									timestamp_now.unix_millis(),
								);
								if !recipe.is_forever && recipe.times >= recipe.max_times {
									Self::set_recipe_done(block_number, *recipe_id);
								}
							},

							Err(e) => {
								log::info!("###### publish_task mail error  {:?}", e);
							},
						};
					},
					Some(Action::MailByLocalServer(revicer, title, body)) => {
						let revicer = match scale_info::prelude::string::String::from_utf8(
							revicer.to_vec(),
						) {
							Ok(v) => v,
							Err(e) => {
								log::info!("###### decode revicer error  {:?}", e);
								continue
							},
						};

						let title =
							match scale_info::prelude::string::String::from_utf8(title.to_vec()) {
								Ok(v) => v,
								Err(e) => {
									log::info!("###### decode title error  {:?}", e);
									continue
								},
							};

						let body =
							match scale_info::prelude::string::String::from_utf8(body.to_vec()) {
								Ok(v) => v,
								Err(e) => {
									log::info!("###### decode body error  {:?}", e);
									continue
								},
							};

						//todo send mail by local docker server url.
						Self::update_recipe_times(
							block_number,
							*recipe_id,
							recipe.times,
							timestamp_now.unix_millis(),
						);
						if !recipe.is_forever && recipe.times >= recipe.max_times {
							Self::set_recipe_done(block_number, *recipe_id);
						}
					},
					Some(Action::Slack(url, message)) => {
						let url = match scale_info::prelude::string::String::from_utf8(url.to_vec())
						{
							Ok(v) => v,
							Err(e) => {
								log::info!("###### decode url error  {:?}", e);
								continue
							},
						};

						let message = match scale_info::prelude::string::String::from_utf8(
							message.to_vec(),
						) {
							Ok(v) => v,
							Err(e) => {
								log::info!("###### decode message error  {:?}", e);
								continue
							},
						};

						let options = scale_info::prelude::format!(
							"node index.js --url={} --message={}",
							url,
							message,
						);

						log::info!("###### publish_task slack options  {:?}", options);

						let _rt = match Self::publish_task(
							"registry.cn-shenzhen.aliyuncs.com/difttt/slack-notify:latest",
							&options,
							3,
						) {
							Ok(_) => {
								log::info!("###### publish_task slack ok");

								Self::update_recipe_times(
									block_number,
									*recipe_id,
									recipe.times,
									timestamp_now.unix_millis(),
								);
								if !recipe.is_forever && recipe.times >= recipe.max_times {
									Self::set_recipe_done(block_number, *recipe_id);
								}
							},
							Err(e) => {
								log::info!("###### publish_task slack error  {:?}", e);
							},
						};
					},
					_ => {},
				}
			}

			Ok(0)
		}

		fn publish_task(
			dockr_url: &str,
			options: &str,
			max_run_num: u64,
		) -> Result<u64, http::Error> {
			let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(10_000));
			let dockr_url = BASE64.encode(dockr_url.as_bytes());
			let options = BASE64.encode(options.as_bytes());

			//let url = "https://reqbin.com/echo/post/json";
			let url = "http://127.0.0.1:8000/".to_owned() +
				&dockr_url.to_owned() +
				"/" + &options.to_owned() +
				"/" + &max_run_num.to_string();

			let request = http::Request::get(&url).add_header("content-type", "application/json");

			let pending = request.deadline(deadline).send().map_err(|e| {
				log::info!("####post pending error: {:?}", e);
				http::Error::IoError
			})?;

			let response = pending.try_wait(deadline).map_err(|e| {
				log::info!("####post response error: {:?}", e);
				http::Error::DeadlineReached
			})??;

			if response.code != 200 {
				log::info!("Unexpected status code: {}", response.code);
				return Err(http::Error::Unknown)
			}

			let body = response.body().collect::<Vec<u8>>();

			// Create a str slice from the body.
			let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
				log::info!("No UTF8 body");
				http::Error::Unknown
			})?;

			if "ok" != body_str {
				log::info!("publish task fail: {}", body_str);
				return Err(http::Error::Unknown)
			}

			Ok(0)
		}

		fn set_recipe_done(block_number: T::BlockNumber, recipe_id: u64) -> Result<u64, Error<T>> {
			if let Some((_, res)) = Signer::<T, T::AuthorityId>::any_account()
				.send_unsigned_transaction(
					// this line is to prepare and return payload
					|account| RecipeDonePayload {
						block_number,
						recipe_id,
						public: account.public.clone(),
					},
					|payload, signature| Call::submit_recipe_done_with_signed_payload {
						recipe_done_payload: payload,
						signature,
					},
				) {
				match res {
					Ok(()) => {
						log::info!("#####unsigned tx with signed payload successfully sent.");
					},
					Err(()) => {
						log::error!("#####sending unsigned tx with signed payload failed.");
					},
				};
			} else {
				// The case of `None`: no account is available for sending
				log::error!("#####No local account available");
			}

			Ok(0)
		}

		fn update_recipe_times(
			block_number: T::BlockNumber,
			recipe_id: u64,
			times: u64,
			timestamp: u64,
		) -> Result<u64, Error<T>> {
			if let Some((_, res)) = Signer::<T, T::AuthorityId>::any_account()
				.send_unsigned_transaction(
					// this line is to prepare and return payload
					|account| RecipeTimesPayload {
						block_number,
						recipe_id,
						times,
						timestamp,
						public: account.public.clone(),
					},
					|payload, signature| Call::submit_recipe_triger_times_with_signed_payload {
						recipe_times_payload: payload,
						signature,
					},
				) {
				match res {
					Ok(()) => {
						log::info!("#####unsigned tx with signed payload successfully sent.");
					},
					Err(()) => {
						log::error!("#####sending unsigned tx with signed payload failed.");
					},
				};
			} else {
				// The case of `None`: no account is available for sending
				log::error!("#####No local account available");
			}

			Ok(0)
		}
	}

	impl<T: Config> BlockNumberProvider for Pallet<T> {
		type BlockNumber = T::BlockNumber;

		fn current_block_number() -> Self::BlockNumber {
			<frame_system::Pallet<T>>::block_number()
		}
	}

	impl<T: Config> TransferProtectInterface<BalanceOf<T>> for Pallet<T> {
		fn get_amout_limit() -> BalanceOf<T> {
			AmountLimit::<T>::get().unwrap_or(Default::default())
		}
		fn get_tx_block_limit() -> u64 {
			TxBlockLimit::<T>::get().unwrap_or(3)
		}
	}
}
