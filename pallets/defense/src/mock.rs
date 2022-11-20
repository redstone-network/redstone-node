use crate as pallet_defense;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64},
};
use frame_system as system;

use sp_core::{sr25519::Signature, H256};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

use pallet_notification::NotificationInfoInterface;
use primitives::{
	custom_call::CustomCallInterface, permission_capture::PermissionCaptureInterface,
	ReserveIdentifier,
};

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		DefenseModule: pallet_defense::{Pallet, Call, Storage, Event<T>},
		NotificationModule: pallet_notification,
		PermissionCaptureModule: pallet_permission_capture,
	}
);

use sp_runtime::{
	testing::{Header, TestXt},
	traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
};

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	// pub const ExistentialDeposit: Balance = 0;
	pub const MaxReserves: u32 = 50;
}
impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ConstU64<1>;

	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ReserveIdentifier;
	type WeightInfo = ();
}

impl pallet_defense::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type AuthorityId = pallet_defense::crypto::OcwAuthId;
	type Call = Call;
	type CustomCallInterface = CustomCall;
	type PermissionCaptureInterface = PermissionCaptureModule;
	type Notification = NotificationModule;
}

type Extrinsic = TestXt<Call, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub struct CustomCall;
impl CustomCallInterface<AccountId, u128> for CustomCall {
	fn call_transfer(to: AccountId, value: u128) -> Vec<u8> {
		let call = Call::DefenseModule(DefenseModule::Call::safe_transfer { to, value });
		call.encode()
	}
}

impl frame_system::offchain::SigningTypes for Test {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
	Call: From<LocalCall>,
{
	type OverarchingCall = Call;
	type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		_public: <Signature as Verify>::Signer,
		_account: AccountId,
		nonce: u64,
	) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
		Some((call, (nonce, ())))
	}
}

impl pallet_notification::Config for Test {
	type Event = Event;
}

impl pallet_permission_capture::Config for Test {
	type Event = Event;
	type MaxFriends = MaxFriends;
	type Currency = Balances;
	type Call = Call;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ts = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> { balances: vec![(0, 1000), (1, 1000), (2, 1000)] }
		.assimilate_storage(&mut ts)
		.unwrap();
	let mut ts = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut ext = sp_io::TestExternalities::new(ts);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
