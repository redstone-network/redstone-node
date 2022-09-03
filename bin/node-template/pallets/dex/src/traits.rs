use primitives::{AccountId, Balance, CurrencyId};
use sp_runtime::DispatchError;
use support::SwapLimit;

//swap token
pub trait SwapTokenInterface<AccountId, Balance, CurrencyId> {
	fn swap_with_different_currency(
		who: &AccountId,
		path: &[CurrencyId],
		limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError>;
}
