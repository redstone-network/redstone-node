#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::Currency, BoundedVec, RuntimeDebug};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::traits::One;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

type FriendsOf<T> = BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxFriends>;

/// An active recovery process.
#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ActiveCapture<BlockNumber, Friends, CaptureStatue> {
	/// The block number when the capture process started.
	created: BlockNumber,
	/// The friends which have vouched so far. Always sorted.
	friends_approve: Friends,
	friends_cancel: Friends,

	execute_proposal_id: u64,
	cancel_proposal_id: u64,

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
	pub multisig_account: AccountId,

	pub approve_votes: u128,
	pub deny_votes: u128,

	pub statue: ProposalStatue,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
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
	pub type ActiveCaptures<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
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

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		NotCaptureable,
		MaxFriends,
		NotStarted,
		AlreadyApproved,
		ProposalNotExist,
		ProposalNotEnd,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.

		#[pallet::weight(0)]
		pub fn create_get_account_permissions(
			origin: OriginFor<T>,  //friends
			account: T::AccountId, //capture owner
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let multisig_account = account.clone(); // todo create a multisig account(all friends is one of the multisig member) for current
										// accout.

			let next_proposal_id = NextProposalId::<T>::get().unwrap_or_default();

			let active_capture = ActiveCapture {
				created: <frame_system::Pallet<T>>::block_number(),
				friends_approve: Default::default(),
				friends_cancel: Default::default(),

				execute_proposal_id: next_proposal_id,
				cancel_proposal_id: Default::default(),
				capture_statue: CaptureStatue::Processing,
			};

			<ActiveCaptures<T>>::insert(&account, &multisig_account, active_capture);

			let proposal = Proposal {
				proposal_type: ProposalType::ExecuteCapture,
				proposer: who.clone(),
				capture_owner: account.clone(),
				multisig_account: multisig_account.clone(),

				approve_votes: Default::default(),
				deny_votes: Default::default(),

				statue: ProposalStatue::Processing,
			};

			Proposals::<T>::insert(next_proposal_id, proposal);
			NextProposalId::<T>::set(Some(next_proposal_id.saturating_add(One::one())));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn cancel_get_account_permissions(
			origin: OriginFor<T>,  //friends
			account: T::AccountId, //capture owner
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let multisig_account = account.clone();

			let _capture_config =
				Self::capture_config(&account).ok_or(Error::<T>::NotCaptureable)?;
			let active_capture =
				Self::active_capture(&account, &multisig_account).ok_or(Error::<T>::NotStarted)?;

			let next_proposal_id = NextProposalId::<T>::get().unwrap_or_default();

			<ActiveCaptures<T>>::insert(
				&account,
				&multisig_account,
				ActiveCapture { cancel_proposal_id: next_proposal_id, ..active_capture },
			);

			let proposal = Proposal {
				proposal_type: ProposalType::Cancel,
				proposer: who.clone(),
				capture_owner: account.clone(),
				multisig_account: multisig_account.clone(),

				approve_votes: Default::default(),
				deny_votes: Default::default(),

				statue: ProposalStatue::Processing,
			};

			Proposals::<T>::insert(next_proposal_id, proposal);
			NextProposalId::<T>::set(Some(next_proposal_id.saturating_add(One::one())));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn vote(
			origin: OriginFor<T>, //friends
			proposal_id: u64,     //capture owner
			_vote: u16,           //0: approve, 1:  deny
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let proposal = Self::proposals(&proposal_id).ok_or(Error::<T>::ProposalNotExist)?;
			ensure!(proposal.statue != ProposalStatue::End, Error::<T>::ProposalNotEnd);

			let capture_config =
				Self::capture_config(&proposal.capture_owner).ok_or(Error::<T>::NotCaptureable)?;

			let mut active_capture =
				Self::active_capture(&proposal.capture_owner, &proposal.multisig_account)
					.ok_or(Error::<T>::NotStarted)?;

			match proposal.proposal_type.clone() {
				ProposalType::ExecuteCapture => {
					match active_capture.friends_approve.binary_search(&who) {
						Ok(_pos) => return Err(Error::<T>::AlreadyApproved.into()),
						Err(pos) => active_capture
							.friends_approve
							.try_insert(pos, who.clone())
							.map_err(|()| Error::<T>::MaxFriends)?,
					}
					<ActiveCaptures<T>>::insert(
						&proposal.capture_owner,
						&proposal.multisig_account,
						&active_capture,
					);

					if capture_config.threshold as usize <= active_capture.friends_approve.len() {
						<ActiveCaptures<T>>::insert(
							&proposal.capture_owner,
							&proposal.multisig_account,
							ActiveCapture {
								capture_statue: CaptureStatue::PermissionTaken,
								..active_capture
							},
						);

						Self::set_proposal_end(active_capture.execute_proposal_id)?;
						Self::set_proposal_end(active_capture.cancel_proposal_id)?;

						<MapPermissionTaken<T>>::insert(&proposal.capture_owner, ());
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
					<ActiveCaptures<T>>::insert(
						&proposal.capture_owner,
						&proposal.multisig_account,
						&active_capture,
					);

					if capture_config.threshold as usize <= active_capture.friends_cancel.len() {
						<ActiveCaptures<T>>::insert(
							&proposal.capture_owner,
							&proposal.multisig_account,
							ActiveCapture {
								capture_statue: CaptureStatue::Canceled,
								..active_capture
							},
						);

						Self::set_proposal_end(active_capture.execute_proposal_id)?;
						Self::set_proposal_end(active_capture.cancel_proposal_id)?;
					}
				},
			}

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn create_capture_config(
			origin: OriginFor<T>, //capture owner
			friends: Vec<T::AccountId>,
			threshold: u16,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let bounded_friends: FriendsOf<T> =
				friends.try_into().map_err(|()| Error::<T>::MaxFriends)?;

			// Create the capture configuration
			let capture_config = CaptureConfig { friends: bounded_friends, threshold };
			// Create the capture configuration storage item
			<Captureable<T>>::insert(&who, capture_config);

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn operational_voting(
			origin: OriginFor<T>, //friends
			_hash: [u8; 32],
			_vote: u16,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn set_proposal_end(proposal_id: u64) -> DispatchResult {
			let proposal = Self::proposals(&proposal_id).ok_or(Error::<T>::ProposalNotExist)?;
			ensure!(proposal.statue != ProposalStatue::End, Error::<T>::ProposalNotEnd);

			Proposals::<T>::insert(
				proposal_id,
				Proposal { statue: ProposalStatue::End, ..proposal },
			);

			Ok(())
		}
	}
}
