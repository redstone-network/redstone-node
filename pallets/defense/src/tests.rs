use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

use sp_core::sr25519::Public;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		// assert_ok!(DefenseModule::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		// assert_eq!(DefenseModule::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		// assert_noop!(DefenseModule::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
	});
}
