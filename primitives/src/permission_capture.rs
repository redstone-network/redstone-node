pub trait PermissionCaptureInterface<AccountId, OpaqueCall, Balance> {
	fn is_account_permission_taken(account: AccountId) -> bool;
	fn has_account_pedding_call(account: AccountId) -> bool;
	fn is_the_same_hash(account: AccountId, call_hash: [u8; 32]) -> bool;
	fn is_call_approved(account: AccountId, call_hash: [u8; 32]) -> bool;
	fn add_call_to_approval_list(
		account: AccountId,
		call_hash: [u8; 32],
		data: OpaqueCall,
		other_deposit: Balance,
	) -> bool;
}
