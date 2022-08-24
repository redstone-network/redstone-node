pub trait TransferProtectInterface<Balance> {
	fn get_amout_limit() -> Balance;
	fn get_tx_block_limit() -> u64;
}
