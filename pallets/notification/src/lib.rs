#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, traits::ConstU32, BoundedVec};
	use frame_system::pallet_prelude::*;
	use pallet_difttt::Action;
	pub use primitives::notification_info::NotificationInfoInterface;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::storage]
	#[pallet::getter(fn map_notify_action)]
	pub(super) type MapNofityAction<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u64, Action<T::AccountId>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		MailCreated(T::AccountId, Vec<u8>, Vec<u8>, Vec<u8>),
		SlackCreated(T::AccountId, Vec<u8>, Vec<u8>),
		DiscordCreated(T::AccountId, Vec<u8>, Vec<u8>, Vec<u8>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// set mail config
		///
		/// - `receiver`: the email receiver
		/// - `title`: the email title
		/// - `body`:  the email body
		#[pallet::weight(0)]
		pub fn set_mail(
			origin: OriginFor<T>,
			receiver: Vec<u8>,
			title: Vec<u8>,
			body: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut bound_receiver = BoundedVec::<u8, ConstU32<128>>::default();
			let mut bound_title = BoundedVec::<u8, ConstU32<128>>::default();
			let mut bound_body = BoundedVec::<u8, ConstU32<256>>::default();

			for x in receiver.iter() {
				let _rt = bound_receiver.try_push(*x);
			}
			for x in title.iter() {
				let _rt = bound_title.try_push(*x);
			}
			for x in body.iter() {
				let _rt = bound_body.try_push(*x);
			}

			let mail = Action::MailWithToken(
				Default::default(),
				Default::default(),
				bound_receiver,
				bound_title,
				bound_body,
			);

			MapNofityAction::<T>::insert(who.clone(), Self::value_in_actions(mail.clone()), mail);
			Self::deposit_event(Event::MailCreated(who, receiver, title, body));

			Ok(())
		}

		/// set slack config
		///
		/// - `hook_url`: the slack hook_url
		/// - `message`: the slack message
		#[pallet::weight(0)]
		pub fn set_slack(
			origin: OriginFor<T>,
			hook_url: Vec<u8>,
			message: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut bound_hook_url = BoundedVec::<u8, ConstU32<256>>::default();
			let mut bound_message = BoundedVec::<u8, ConstU32<256>>::default();

			for x in hook_url.iter() {
				let _rt = bound_hook_url.try_push(*x);
			}
			for x in message.iter() {
				let _rt = bound_message.try_push(*x);
			}

			let slack = Action::Slack(bound_hook_url, bound_message);
			MapNofityAction::<T>::insert(who.clone(), Self::value_in_actions(slack.clone()), slack);

			Self::deposit_event(Event::SlackCreated(who, hook_url, message));

			Ok(())
		}

		/// set discord config
		///
		/// - `hook_url`: the discord hook_url
		/// - `user`: the discord user name
		/// - `content`: the discord message content
		#[pallet::weight(0)]
		pub fn set_discord(
			origin: OriginFor<T>,
			hook_url: Vec<u8>,
			user: Vec<u8>,
			content: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut bound_hook_url = BoundedVec::<u8, ConstU32<256>>::default();
			let mut bound_user = BoundedVec::<u8, ConstU32<64>>::default();
			let mut bound_content = BoundedVec::<u8, ConstU32<256>>::default();

			for x in hook_url.iter() {
				let _rt = bound_hook_url.try_push(*x);
			}
			for x in user.iter() {
				let _rt = bound_user.try_push(*x);
			}
			for x in content.iter() {
				let _rt = bound_content.try_push(*x);
			}

			let discord = Action::Discord(bound_hook_url, bound_user, bound_content);
			MapNofityAction::<T>::insert(
				who.clone(),
				Self::value_in_actions(discord.clone()),
				discord,
			);

			Self::deposit_event(Event::DiscordCreated(who, hook_url, user, content));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn value_in_actions(a: Action<T::AccountId>) -> u64 {
			match a {
				Action::MailWithToken(..) => 0,
				Action::BuyToken(..) => 1,
				Action::MailByLocalServer(..) => 2,
				Action::Slack(..) => 3,
				Action::Discord(..) => 4,
			}
		}
	}

	impl<T: Config> NotificationInfoInterface<T::AccountId, Action<T::AccountId>> for Pallet<T> {
		fn get_mail_config_action(account: T::AccountId) -> Option<Action<T::AccountId>> {
			MapNofityAction::<T>::get(account, 0)
		}
		fn get_slack_config_action(account: T::AccountId) -> Option<Action<T::AccountId>> {
			MapNofityAction::<T>::get(account, 3)
		}
		fn get_discord_config_action(account: T::AccountId) -> Option<Action<T::AccountId>> {
			MapNofityAction::<T>::get(account, 4)
		}
	}
}
