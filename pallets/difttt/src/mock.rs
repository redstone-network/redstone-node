use super::*;
use crate as pallet_difttt;
use frame_support::{
	pallet_prelude::DispatchResult,
	parameter_types,
	traits::{ConstU16, ConstU64, Nothing},
	PalletId,
};
use frame_system as system;

use orml_traits::MultiReservableCurrency;
use primitives::{
	evm::EvmAddress, Balance, BlockNumber, CurrencyId, ReserveIdentifier, TokenSymbol,
};
use sp_core::{sr25519::Signature, H256};
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{BlakeTwo256, Extrinsic as ExtrinsicT, IdentifyAccount, IdentityLookup, Verify},
};
use support::{DEXIncentives, Erc20InfoMapping};

pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		DiftttModule: pallet_difttt::{Pallet, Call, Storage, Event<T>},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		Balances: pallet_balances::{Pallet, Storage, Event<T>, Config<T>},
		Currencies: orml_currencies::{Pallet, Call},
		Dex: pallet_dex::{Pallet, Storage, Call, Event<T>, Config<T>},
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
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<3>;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 0;
	pub const MaxReserves: u32 = 50;
}
impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	// type AccountStore = pallet_balances::AccountData<u128>;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ReserveIdentifier;
	type WeightInfo = ();
}

orml_traits::parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		0
	};
}

impl orml_tokens::Config for Test {
	type Amount = Amount;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type DustRemovalWhitelist = Nothing;
	type Event = Event;
	type ExistentialDeposits = ExistentialDeposits;
	type MaxLocks = ();
	type MaxReserves = ();
	type OnDust = ();
	type ReserveIdentifier = ReserveIdentifier;
	type WeightInfo = ();
	type TransferProtectInterface = DiftttModule;
	type OnNewTokenAccount = ();
	type OnKilledTokenAccount = ();
}

pub type Amount = i128;
parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::ACA);
}

// pub type AdaptedBasicCurrency = BasicCurrencyAdapter<Runtime, PalletBalances, i64, u64>;

impl orml_currencies::Config for Test {
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type MultiCurrency = Tokens;
	type NativeCurrency =
		orml_currencies::BasicCurrencyAdapter<Test, Balances, Amount, BlockNumber>; //BasicCurrencyAdapter<T, Currency, Amount, Moment>;
	type WeightInfo = ();
}

parameter_types! {
	pub const UnsignedPriority: u64 = 1;
}

impl pallet_difttt::Config for Test {
	type Event = Event;
	type TimeProvider = pallet_timestamp::Pallet<Test>;
	type Call = Call;
	type UnsignedPriority = UnsignedPriority;
	type WeightInfo = ();

	type Currency = Currencies;
	// type SwapToken = Dex;
	type AuthorityId = pallet_difttt::crypto::TestAuthId;
}

parameter_types! {
	pub const GetExchangeFee: (u32, u32) = (3, 1000);	// 0.3%
	pub const ExtendedProvisioningBlocks: BlockNumber = 2 * DAYS;
	pub const TradingPathLimit: u32 = 4;
	pub const DEXPalletId: PalletId = PalletId(*b"aca/dexm");
}

pub struct MockErc20InfoMapping;

impl Erc20InfoMapping for MockErc20InfoMapping {
	fn name(_currency_id: CurrencyId) -> Option<Vec<u8>> {
		None
	}

	fn symbol(_currency_id: CurrencyId) -> Option<Vec<u8>> {
		None
	}

	fn decimals(_currency_id: CurrencyId) -> Option<u8> {
		None
	}

	fn encode_evm_address(_v: CurrencyId) -> Option<EvmAddress> {
		None
	}

	fn decode_evm_address(_v: EvmAddress) -> Option<CurrencyId> {
		None
	}
}

pub struct MockDEXIncentives;
impl DEXIncentives<AccountId, CurrencyId, Balance> for MockDEXIncentives {
	fn do_deposit_dex_share(
		who: &AccountId,
		lp_currency_id: CurrencyId,
		amount: Balance,
	) -> DispatchResult {
		Tokens::reserve(lp_currency_id, who, amount)
	}

	fn do_withdraw_dex_share(
		who: &AccountId,
		lp_currency_id: CurrencyId,
		amount: Balance,
	) -> DispatchResult {
		let _ = Tokens::unreserve(lp_currency_id, who, amount);
		Ok(())
	}
}

/// Configure the pallet-dex in pallets/dex.
impl pallet_dex::Config for Test {
	type Event = Event;
	type Currency = Currencies;
	type GetExchangeFee = GetExchangeFee;
	type TradingPathLimit = TradingPathLimit;
	type PalletId = DEXPalletId;
	type Erc20InfoMapping = MockErc20InfoMapping;
	type DEXIncentives = MockDEXIncentives;
	type WeightInfo = ();
	// type ListingOrigin = EnsureRootOrHalfGeneralCouncil;
	type ExtendedProvisioningBlocks = ExtendedProvisioningBlocks;
	type OnLiquidityPoolUpdated = ();
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
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
