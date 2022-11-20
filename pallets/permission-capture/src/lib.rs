#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	traits::{Currency, Get, WrapperKeepOpaque},
	weights::{GetDispatchInfo, PostDispatchInfo},
	BoundedVec, RuntimeDebug,
};
use frame_system::RawOrigin;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::traits::{Dispatchable, One};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

type FriendsOf<T> = BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxFriends>;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// An active recovery process.
#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ActiveCapture<BlockNumber, Friends, CaptureStatue> {
	/// The block number when the capture process started.
	created: BlockNumber,
	/// The friends which have vouched so far. Always sorted.
	friends_approve: Friends,
	friends_cancel: Friends,

	execute_proposal_id: u64,
	cancel_proposal_id: Option<u64>,

	capture_statue: CaptureStatue,
}

/// Configuration for recovering an account.
#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CaptureConfig<Friends> {
	/// The list of friends which can help recover an account. Always sorted.
	friends: Friends,
	/// The number of approving friends needed to recover an account.
	threshold: u16,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ProposalType {
	ExecuteCapture,
	Cancel,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum CaptureStatue {
	Processing,
	Canceled,
	PermissionTaken,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ProposalStatue {
	Processing,
	End,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Proposal<AccountId> {
	pub proposal_type: ProposalType,
	pub proposer: AccountId,
	pub capture_owner: AccountId,

	pub approve_votes: u128,
	pub deny_votes: u128,

	pub statue: ProposalStatue,
}

/// An open multisig operation.
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ActiveCall<Friends, OpaqueCall, AccountId, Balance> {
	/// The approvals achieved so far, including the depositor. Always sorted.
	approvals: Friends,
	denys: Friends,
	info: (OpaqueCall, AccountId, Balance),
}

type OpaqueCall<T> = WrapperKeepOpaque<<T as Config>::Call>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	pub use primitives::permission_capture::PermissionCaptureInterface;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type MaxFriends: Get<u32>;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The overarching call type.
		type Call: Parameter
			+ Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
			+ GetDispatchInfo
			+ From<frame_system::Call<Self>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn map_permission_taken)]
	pub(super) type MapPermissionTaken<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, ()>;

	/// The set of recoverable accounts and their capture configuration.
	#[pallet::storage]
	#[pallet::getter(fn capture_config)]
	pub type Captureable<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, CaptureConfig<FriendsOf<T>>>;

	/// Active recovery attempts.
	///
	/// First account is the account to be recovered, and the second account
	/// is the user trying to recover the account.
	#[pallet::storage]
	#[pallet::getter(fn active_capture)]
	pub type ActiveCaptures<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		ActiveCapture<T::BlockNumber, FriendsOf<T>, CaptureStatue>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn next_proposal_id)]
	pub type NextProposalId<T: Config> = StorageValue<_, u64>;

	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config> = StorageMap<_, Twox64Concat, u64, Proposal<T::AccountId>>;

	#[pallet::storage]
	pub type OwnerCalls<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, [u8; 32]>;
	#[pallet::storage]
	pub type ActiveCalls<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Blake2_128Concat,
		[u8; 32],
		ActiveCall<FriendsOf<T>, OpaqueCall<T>, T::AccountId, BalanceOf<T>>,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		NewCapture(u64, T::AccountId, T::AccountId),
		NewCancelCapture(u64, T::AccountId, T::AccountId),
		ProposalOneFriendApproval(u64, T::AccountId, T::AccountId),
		CaptureExecuted(T::AccountId),
		CaptureCancelled(T::AccountId),
		NewCaptureConfig(T::AccountId, Vec<T::AccountId>, u16),
		CallExecuted(T::AccountId, [u8; 32], OpaqueCall<T>),
		CallCancelled(T::AccountId, [u8; 32], OpaqueCall<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NotCaptureable,
		MaxFriends,
		NotStarted,
		AlreadyApproved,
		ProposalNotExist,
		ProposalMustProcessing,
		MustProposalByFriends,
		ProposalAlreadyCreated,
		MustVoteByFriends,
		ActiveCallNotExist,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.

		/// friends create a get account permission proposal
		///
		/// - `account`: the permission owner
		#[pallet::weight(0)]
		pub fn create_get_account_permissions(
			origin: OriginFor<T>,  //friends
			account: T::AccountId, //capture owner
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let capture_config =
				Self::capture_config(&account).ok_or(Error::<T>::NotCaptureable)?;
			ensure!(capture_config.friends.contains(&who), Error::<T>::MustProposalByFriends);
			ensure!(
				!<ActiveCaptures<T>>::contains_key(&account),
				Error::<T>::ProposalAlreadyCreated
			);

			let next_proposal_id = NextProposalId::<T>::get().unwrap_or_default();

			let active_capture = ActiveCapture {
				created: <frame_system::Pallet<T>>::block_number(),
				friends_approve: Default::default(),
				friends_cancel: Default::default(),

				execute_proposal_id: next_proposal_id,
				cancel_proposal_id: Default::default(),
				capture_statue: CaptureStatue::Processing,
			};

			<ActiveCaptures<T>>::insert(&account, active_capture);

			let proposal = Proposal {
				proposal_type: ProposalType::ExecuteCapture,
				proposer: who.clone(),
				capture_owner: account.clone(),

				approve_votes: Default::default(),
				deny_votes: Default::default(),

				statue: ProposalStatue::Processing,
			};

			Proposals::<T>::insert(next_proposal_id, proposal);
			NextProposalId::<T>::set(Some(next_proposal_id.saturating_add(One::one())));

			Self::deposit_event(Event::NewCapture(next_proposal_id, account, who));

			Ok(())
		}

		/// friends cancel a get account permission proposal
		///
		/// - `account`: the permission owner
		#[pallet::weight(0)]
		pub fn cancel_get_account_permissions(
			origin: OriginFor<T>,  //friends
			account: T::AccountId, //capture owner
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let capture_config =
				Self::capture_config(&account).ok_or(Error::<T>::NotCaptureable)?;

			let active_capture = Self::active_capture(&account).ok_or(Error::<T>::NotStarted)?;
			ensure!(capture_config.friends.contains(&who), Error::<T>::MustProposalByFriends);

			let next_proposal_id = NextProposalId::<T>::get().unwrap_or_default();

			<ActiveCaptures<T>>::insert(
				&account,
				ActiveCapture { cancel_proposal_id: Some(next_proposal_id), ..active_capture },
			);

			let proposal = Proposal {
				proposal_type: ProposalType::Cancel,
				proposer: who.clone(),
				capture_owner: account.clone(),

				approve_votes: Default::default(),
				deny_votes: Default::default(),

				statue: ProposalStatue::Processing,
			};

			Proposals::<T>::insert(next_proposal_id, proposal);
			NextProposalId::<T>::set(Some(next_proposal_id.saturating_add(One::one())));

			Self::deposit_event(Event::NewCancelCapture(next_proposal_id, account, who));

			Ok(())
		}

		/// friends vote the get account permission proposal, or cancel get account permission
		/// proposal
		///
		/// - `proposal_id`: the proposal id
		/// - `_vote`: the vote type, 0: approve, 1:  deny, current default is 0.
		#[pallet::weight(0)]
		pub fn vote(
			origin: OriginFor<T>, //friends
			proposal_id: u64,     //proposal id
			_vote: u16,           //0: approve, 1:  deny
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let proposal = Self::proposals(&proposal_id).ok_or(Error::<T>::ProposalNotExist)?;
			ensure!(proposal.statue != ProposalStatue::End, Error::<T>::ProposalMustProcessing);

			let capture_config =
				Self::capture_config(&proposal.capture_owner).ok_or(Error::<T>::NotCaptureable)?;
			ensure!(capture_config.friends.contains(&who), Error::<T>::MustVoteByFriends);

			let mut active_capture =
				Self::active_capture(&proposal.capture_owner).ok_or(Error::<T>::NotStarted)?;

			match proposal.proposal_type.clone() {
				ProposalType::ExecuteCapture => {
					match active_capture.friends_approve.binary_search(&who) {
						Ok(_pos) => return Err(Error::<T>::AlreadyApproved.into()),
						Err(pos) => active_capture
							.friends_approve
							.try_insert(pos, who.clone())
							.map_err(|()| Error::<T>::MaxFriends)?,
					}
					<ActiveCaptures<T>>::insert(&proposal.capture_owner, &active_capture);
					<Proposals<T>>::insert(
						&proposal_id,
						Proposal { approve_votes: &proposal.approve_votes + 1, ..proposal.clone() },
					);

					if capture_config.threshold as usize <= active_capture.friends_approve.len() {
						<ActiveCaptures<T>>::insert(
							&proposal.capture_owner,
							ActiveCapture {
								capture_statue: CaptureStatue::PermissionTaken,
								..active_capture
							},
						);

						Self::set_proposal_end(active_capture.execute_proposal_id)?;
						if active_capture.cancel_proposal_id.is_some() {
							Self::set_proposal_end(
								active_capture.cancel_proposal_id.unwrap_or_default(),
							)?;
						}

						<MapPermissionTaken<T>>::insert(&proposal.capture_owner, ());

						Self::deposit_event(Event::CaptureExecuted(proposal.capture_owner.clone()));
					}
				},
				ProposalType::Cancel => {
					match active_capture.friends_cancel.binary_search(&who) {
						Ok(_pos) => return Err(Error::<T>::AlreadyApproved.into()),
						Err(pos) => active_capture
							.friends_cancel
							.try_insert(pos, who.clone())
							.map_err(|()| Error::<T>::MaxFriends)?,
					}
					<ActiveCaptures<T>>::insert(&proposal.capture_owner, &active_capture);
					<Proposals<T>>::insert(
						&proposal_id,
						Proposal { approve_votes: &proposal.approve_votes + 1, ..proposal.clone() },
					);

					if capture_config.threshold as usize <= active_capture.friends_cancel.len() {
						<ActiveCaptures<T>>::insert(
							&proposal.capture_owner,
							ActiveCapture {
								capture_statue: CaptureStatue::Canceled,
								..active_capture
							},
						);

						Self::set_proposal_end(active_capture.execute_proposal_id)?;
						if active_capture.cancel_proposal_id.is_some() {
							Self::set_proposal_end(
								active_capture.cancel_proposal_id.unwrap_or_default(),
							)?;
						}

						Self::deposit_event(Event::CaptureCancelled(
							proposal.capture_owner.clone(),
						));
					}
				},
			}

			Self::deposit_event(Event::ProposalOneFriendApproval(
				proposal_id,
				proposal.capture_owner,
				who,
			));

			Ok(())
		}

		/// create capture config
		///
		/// - `friends`: the permission owner's friends
		/// - `threshold`: the permission threshold to execute
		#[pallet::weight(0)]
		pub fn create_capture_config(
			origin: OriginFor<T>, //capture owner
			friends: Vec<T::AccountId>,
			threshold: u16,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let bounded_friends: FriendsOf<T> =
				friends.clone().try_into().map_err(|()| Error::<T>::MaxFriends)?;

			// Create the capture configuration
			let capture_config = CaptureConfig { friends: bounded_friends, threshold };
			// Create the capture configuration storage item
			<Captureable<T>>::insert(&who, capture_config);

			Self::deposit_event(Event::NewCaptureConfig(who, friends, threshold));

			Ok(())
		}

		/// vote for call hash of permission owner's
		///
		/// - `account`: the permission owner
		/// - `hash`: the call hash of permission
		/// - `vote`: the vote type, 0: approve, 1:  deny,
		#[pallet::weight(0)]
		pub fn operational_voting(
			origin: OriginFor<T>, //friends
			account: T::AccountId,
			hash: [u8; 32],
			vote: u16,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let capture_config =
				Self::capture_config(&account).ok_or(Error::<T>::NotCaptureable)?;
			ensure!(capture_config.friends.contains(&who), Error::<T>::MustVoteByFriends);

			ensure!(
				<ActiveCalls<T>>::contains_key(account.clone(), hash),
				Error::<T>::ActiveCallNotExist
			);

			let mut active_call = ActiveCalls::<T>::get(account.clone(), hash).unwrap();

			match vote {
				0 => {
					match active_call.approvals.binary_search(&who) {
						Ok(_pos) => return Err(Error::<T>::AlreadyApproved.into()),
						Err(pos) => active_call
							.approvals
							.try_insert(pos, who.clone())
							.map_err(|()| Error::<T>::MaxFriends)?,
					}
					<ActiveCalls<T>>::insert(account.clone(), hash, &active_call);

					if let Some(call) = active_call.info.0.try_decode() {
						if capture_config.threshold as usize <= active_call.approvals.len() {
							call.dispatch(RawOrigin::Signed(account.clone()).into())
								.map_err(|x| x.error)?;
							Self::clear_call(account.clone(), &hash);
							Self::deposit_event(Event::CallExecuted(
								account.clone(),
								hash,
								active_call.info.0,
							));
						}
					}
				},
				1 => {
					match active_call.denys.binary_search(&who) {
						Ok(_pos) => return Err(Error::<T>::AlreadyApproved.into()),
						Err(pos) => active_call
							.denys
							.try_insert(pos, who.clone())
							.map_err(|()| Error::<T>::MaxFriends)?,
					}
					<ActiveCalls<T>>::insert(account.clone(), hash, &active_call);

					if capture_config.threshold as usize <= active_call.denys.len() {
						Self::clear_call(account.clone(), &hash);

						Self::deposit_event(Event::CallCancelled(
							account.clone(),
							hash,
							active_call.info.0,
						));
					}
				},
				_ => {},
			}

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn set_proposal_end(proposal_id: u64) -> DispatchResult {
			let proposal = Self::proposals(&proposal_id).ok_or(Error::<T>::ProposalNotExist)?;
			ensure!(proposal.statue != ProposalStatue::End, Error::<T>::ProposalMustProcessing);

			Proposals::<T>::insert(
				proposal_id,
				Proposal { statue: ProposalStatue::End, ..proposal },
			);

			Ok(())
		}

		fn clear_call(account: T::AccountId, hash: &[u8; 32]) {
			let _owner_call = OwnerCalls::<T>::take(account.clone());
			let _active_call = ActiveCalls::<T>::take(account.clone(), hash);
		}
	}

	impl<T: Config> PermissionCaptureInterface<T::AccountId, OpaqueCall<T>, BalanceOf<T>>
		for Pallet<T>
	{
		fn is_account_permission_taken(account: T::AccountId) -> bool {
			<MapPermissionTaken<T>>::contains_key(account)
		}

		fn has_account_pedding_call(account: T::AccountId) -> bool {
			<OwnerCalls<T>>::contains_key(account)
		}

		fn is_the_same_hash(account: T::AccountId, call_hash: [u8; 32]) -> bool {
			<ActiveCalls<T>>::contains_key(account, call_hash)
		}

		fn is_call_approved(account: T::AccountId, call_hash: [u8; 32]) -> bool {
			if <Captureable<T>>::contains_key(account.clone()) {
				let capture_config = Captureable::<T>::get(account.clone()).unwrap_or_default();
				if <ActiveCalls<T>>::contains_key(account.clone(), call_hash) {
					let active_call = ActiveCalls::<T>::get(account.clone(), call_hash).unwrap();

					return capture_config.threshold as usize <= active_call.approvals.len();
				}
			}

			false
		}

		fn add_call_to_approval_list(
			account: T::AccountId,
			call_hash: [u8; 32],
			data: OpaqueCall<T>,
			other_deposit: BalanceOf<T>,
		) -> bool {
			<OwnerCalls<T>>::insert(&account, &call_hash);
			<ActiveCalls<T>>::insert(
				&account,
				call_hash,
				ActiveCall {
					approvals: FriendsOf::<T>::default(),
					denys: FriendsOf::<T>::default(),
					info: (data, account.clone(), other_deposit),
				},
			);

			return true;
		}
	}
}
