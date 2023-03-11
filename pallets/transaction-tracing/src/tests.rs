use super::*;
use crate::{mock::*, Error};

use frame_support::{assert_noop, assert_ok};

use mock::Event;
use sp_core::sr25519::Public;

#[test]
fn set_tracing_account_should_work() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);

		assert_ok!(TxTracingModule::set_tracing_account(Origin::signed(signer)));

		System::assert_has_event(Event::TxTracingModule(crate::Event::TrancingAccountSet(
			signer.clone(),
		)));
	});
}

#[test]
fn set_tracing_account_would_fail_when_account_has_been_set() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);

		assert_ok!(TxTracingModule::set_tracing_account(Origin::signed(signer)));

		System::assert_has_event(Event::TxTracingModule(crate::Event::TrancingAccountSet(
			signer.clone(),
		)));

		assert_noop!(
			TxTracingModule::set_tracing_account(Origin::signed(signer)),
			Error::<Test>::TracingAccountHasBeenSet
		);
	});
}
