pub trait CustomCallInterface<AccountId, Balance> {
	fn call_transfer(to: AccountId, value: Balance) -> Vec<u8>;
}
