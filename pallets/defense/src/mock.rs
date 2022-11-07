use crate as pallet_defense;
use frame_support::traits::{ConstU16, ConstU64};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		DefenseModule: pallet_defense,
	}
);

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

// 指明类型
type Balance = u64;
// 实现引入的pallet
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
}

impl pallet_defense::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type AuthorityId = pallet_defense::crypto::TestAuthId;
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
