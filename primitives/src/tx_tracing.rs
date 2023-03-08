pub trait AccountStatusInfo<AccountId> {
	fn get_account_status(account: AccountId) -> Option<bool>;
}
