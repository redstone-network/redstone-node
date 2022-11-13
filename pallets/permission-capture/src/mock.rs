use crate as pallet_permission_capture;
use codec::Encode;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64},
};
use frame_system as system;
use pallet_defense::{Call as DefenseCall, Error as DefenseError};
use pallet_difttt::crypto::TestAuthId;
use primitives::custom_call::CustomCallInterface;
use sp_core::{
	sr25519::{Public, Signature},
	H256,
};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type OpaqueCall = super::OpaqueCall<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances,
		PermissionCaptureModule: pallet_permission_capture,
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
	type AccountId = sp_core::sr25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u128;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

pub struct CustomCall;
impl CustomCallInterface<AccountId, u128> for CustomCall {
	fn call_transfer(to: AccountId, value: u128) -> Vec<u8> {
		let call = Call::DefenseModule(DefenseCall::safe_transfer { to, value });
		call.encode()
	}
}

impl pallet_defense::Config for Test {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type AuthorityId = TestAuthId;
	type PermissionCaptureInterface = PermissionCaptureModule;
	type CustomCallInterface = CustomCall;
}

parameter_types! {
	pub const MaxFriends: u32 = 128;
}

impl pallet_permission_capture::Config for Test {
	type Event = Event;
	type MaxFriends = MaxFriends;
	type Currency = Balances;
	type Call = Call;
}

type Extrinsic = TestXt<Call, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

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

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(Public::from_raw([0; 32]), 100),
			(Public::from_raw([1; 32]), 100),
			(Public::from_raw([2; 32]), 100),
			(Public::from_raw([3; 32]), 100),
			(Public::from_raw([4; 32]), 100),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn call_transfer(to: sp_core::sr25519::Public, value: u128) -> Call {
	Call::DefenseModule(DefenseCall::safe_transfer { to, value })
}
