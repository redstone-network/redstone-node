pub trait PermissionCaptureInterface<AccountId> {
	fn IsAccountPermissionTaken(account: AccountId) -> bool;
	fn HasAccountPeddingCall(account: AccountId) -> bool;
	fn AddCallToApprovalList(account: AccountId, call_hash: [u8; 32]) -> bool;
}
