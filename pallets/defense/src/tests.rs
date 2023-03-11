use super::*;
use crate::{mock::*, Error};

use frame_support::{assert_noop, assert_ok};

use mock::Event;
use pallet_tx_tracing::TrancedAccountPayload;
use sp_core::sr25519::{Public, Signature};

#[test]
fn set_transfer_limitation_should_work() {
	new_test_ext().execute_with(|| {
		let amount_limit = TransferLimitation::AmountLimit(1000);
		let frequency_limit = TransferLimitation::FrequencyLimit(5, 100);
		let signer = Public::from_raw([0; 32]);

		// assert
		assert_ok!(DefenseModule::set_transfer_limitation(Origin::signed(signer), amount_limit));
		assert_ok!(DefenseModule::set_transfer_limitation(Origin::signed(signer), frequency_limit));

		// assert transfer limit owner
		assert_eq!(TransferLimitationOwner::<Test>::get(signer, 1), Some(amount_limit));
		assert_eq!(TransferLimitationOwner::<Test>::get(signer, 2), Some(frequency_limit));

		// assert successful events
		System::assert_has_event(Event::DefenseModule(crate::Event::TransferAmountLimitSet(
			signer,
			amount_limit,
		)));
		System::assert_has_event(Event::DefenseModule(crate::Event::TransferFrequencyLimitSet(
			signer,
			frequency_limit,
		)));
	});
}

#[test]
fn set_freeze_configuration_should_work() {
	new_test_ext().execute_with(|| {
		let freeze_account_for_some_time = FreezeConfiguration::TimeFreeze(120);
		let freeze_account_permanent = FreezeConfiguration::AccountFreeze(true);
		let signer = Public::from_raw([0; 32]);

		// assert
		assert_ok!(DefenseModule::set_freeze_configuration(
			Origin::signed(signer),
			freeze_account_for_some_time.clone()
		));
		assert_ok!(DefenseModule::set_freeze_configuration(
			Origin::signed(signer),
			freeze_account_permanent.clone()
		));

		assert_eq!(
			FreezeConfigurationOwner::<Test>::get(&signer, 1),
			Some(freeze_account_for_some_time.clone())
		);

		assert_eq!(
			FreezeConfigurationOwner::<Test>::get(signer, 2),
			Some(freeze_account_permanent.clone())
		);

		// assert successful events
		System::assert_has_event(Event::DefenseModule(crate::Event::FreezeTimeSet(
			signer,
			freeze_account_for_some_time.clone(),
		)));
		System::assert_has_event(Event::DefenseModule(crate::Event::FreezeAccountSet(
			signer,
			freeze_account_permanent.clone(),
		)));
	});
}

#[test]
fn safe_transfer_should_work() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let to = Public::from_raw([1; 32]);
		// assert
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 1));

		let amount_limit = TransferLimitation::AmountLimit(100);
		let frequency_limit = TransferLimitation::FrequencyLimit(10, 1000);

		// assert
		assert_ok!(DefenseModule::set_transfer_limitation(Origin::signed(signer), amount_limit));

		// assert
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));

		assert_ok!(DefenseModule::set_transfer_limitation(Origin::signed(signer), frequency_limit));
		// assert
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));

		// System::assert_has_event(Event::DefenseModule(crate::Event::TransferSuccess(
		// 	signer, to, 3,
		// )));
	});
}

#[test]
fn set_transfer_limitation_should_fail_when_amount_limit_has_set() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let amount_limit = TransferLimitation::AmountLimit(1000);
		let update_amount_limit = TransferLimitation::AmountLimit(2000);

		// assert
		assert_ok!(DefenseModule::set_transfer_limitation(Origin::signed(signer), amount_limit));

		assert_noop!(
			DefenseModule::set_transfer_limitation(Origin::signed(signer), update_amount_limit),
			Error::<Test>::TransferAmountLimitHasSet
		);
	});
}

#[test]
fn set_transfer_limitation_should_fail_when_frequency_limit_has_set() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);

		let frequency_limit = TransferLimitation::FrequencyLimit(5, 100);
		let update_frequency_limit = TransferLimitation::FrequencyLimit(100, 100);

		// assert
		assert_ok!(DefenseModule::set_transfer_limitation(Origin::signed(signer), frequency_limit));

		assert_noop!(
			DefenseModule::set_transfer_limitation(Origin::signed(signer), update_frequency_limit),
			Error::<Test>::TransferFrequencyLimitHasSet
		);
	});
}

#[test]
fn set_freeze_configuration_should_fail_when_freeze_time_has_set() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let mut freeze_account_for_some_time = FreezeConfiguration::TimeFreeze(120);

		assert_ok!(DefenseModule::set_freeze_configuration(
			Origin::signed(signer),
			freeze_account_for_some_time.clone()
		));

		freeze_account_for_some_time = FreezeConfiguration::TimeFreeze(240);

		assert_noop!(
			DefenseModule::set_freeze_configuration(
				Origin::signed(signer),
				freeze_account_for_some_time.clone()
			),
			Error::<Test>::FreezeTimeHasSet
		);
	});
}

#[test]
fn set_freeze_configuration_should_fail_when_freeze_account_has_set() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let mut freeze_account_permanent = FreezeConfiguration::AccountFreeze(true);

		assert_ok!(DefenseModule::set_freeze_configuration(
			Origin::signed(signer),
			freeze_account_permanent.clone()
		));

		freeze_account_permanent = FreezeConfiguration::AccountFreeze(false);

		assert_noop!(
			DefenseModule::set_freeze_configuration(
				Origin::signed(signer),
				freeze_account_permanent.clone()
			),
			Error::<Test>::FreezeAccountHasSet
		);
	});
}

#[test]
fn safe_transfer_should_fail_when_transfer_value_is_larger_than_set_amount() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);

		let amount_limit = TransferLimitation::AmountLimit(50);

		assert_ok!(DefenseModule::set_transfer_limitation(Origin::signed(signer), amount_limit));

		// assert_noop!(
		// 	DefenseModule::safe_transfer(Origin::signed(signer), to, 51),
		// 	Error::<Test>::TransferValueTooLarge
		// );
	});
}

#[test]
fn safe_transfer_should_fail_when_transfer_times_is_more_than_set_times() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let to = Public::from_raw([1; 32]);

		let times_limit = TransferLimitation::FrequencyLimit(3, 100);

		assert_ok!(DefenseModule::set_transfer_limitation(Origin::signed(signer), times_limit));
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));

		// assert_noop!(
		// 	DefenseModule::safe_transfer(Origin::signed(signer), to, 3),
		// 	Error::<Test>::TransferTimesTooMany
		// );
	});
}

#[test]
fn safe_transfer_should_fail_when_account_freeze_temporary() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let to = Public::from_raw([1; 32]);

		let amount_limit = TransferLimitation::AmountLimit(50);

		assert_ok!(DefenseModule::set_transfer_limitation(Origin::signed(signer), amount_limit));

		let freeze_account_for_some_time = FreezeConfiguration::TimeFreeze(120);

		assert_ok!(DefenseModule::set_freeze_configuration(
			Origin::signed(signer),
			freeze_account_for_some_time.clone()
		));

		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 60));

		assert_noop!(
			DefenseModule::safe_transfer(Origin::signed(signer), to, 10),
			Error::<Test>::AccountHasBeenFrozenTemporary
		);
	});
}

#[test]
fn safe_transfer_should_fail_when_account_freeze_permanent() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let to = Public::from_raw([1; 32]);

		let times_limit = TransferLimitation::FrequencyLimit(3, 100);

		assert_ok!(DefenseModule::set_transfer_limitation(Origin::signed(signer), times_limit));

		let freeze_account_for_some_time = FreezeConfiguration::TimeFreeze(120);
		let freeze_account_permanent = FreezeConfiguration::AccountFreeze(true);

		assert_ok!(DefenseModule::set_freeze_configuration(
			Origin::signed(signer),
			freeze_account_for_some_time.clone()
		));

		assert_ok!(DefenseModule::set_freeze_configuration(
			Origin::signed(signer),
			freeze_account_permanent.clone()
		));

		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));
		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));

		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));

		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 3));

		assert_noop!(
			DefenseModule::safe_transfer(Origin::signed(signer), to, 3),
			Error::<Test>::AccountHasBeenFrozenPermanent
		);
	});
}

#[test]
fn freeze_account_should_work() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);

		assert_ok!(DefenseModule::freeze_account(Origin::signed(signer), true));

		assert_eq!(BlockAccount::<Test>::get(signer), Some(true));

		System::assert_has_event(Event::DefenseModule(crate::Event::FreezeAccountPermanent(
			signer,
		)));
	})
}

#[test]
fn safe_transfer_should_fail_when_amount_is_too_large_without_risk_management() {
	new_test_ext().execute_with(|| {
		let signer = Public::from_raw([0; 32]);
		let amount_limit = TransferLimitation::AmountLimit(50);
		let to = Public::from_raw([1; 32]);

		// assert
		assert_ok!(DefenseModule::set_transfer_limitation(Origin::signed(signer), amount_limit));

		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 51));

		assert_eq!(DefaultFreeze::<Test>::get(signer), Some(true));

		assert_noop!(
			DefenseModule::safe_transfer(Origin::signed(signer), to, 10),
			Error::<Test>::AccountHasBeenFrozenTemporary
		);
	});
}

// #[test]
// fn safe_transfer_should_fail_when_transaction_is_abnormal() {
// 	new_test_ext().execute_with(|| {
// 		let signer = Public::from_raw([0; 32]);

// 		let to = Public::from_raw([1; 32]);

// 		assert_ok!(TxTracingModule::set_tracing_account(Origin::signed(signer)));

// 		let _s = Signature([0_u8; 64]);

// 		let traced_account_payload =
// 			TrancedAccountPayload { account: signer.clone(), public: signer };

// 		assert_ok!(TxTracingModule::set_account_as_abnormal(
// 			Origin::none(),
// 			traced_account_payload,
// 			_s
// 		));

// 		assert_ok!(DefenseModule::safe_transfer(Origin::signed(signer), to, 51));

// 		assert_eq!(DefaultFreeze::<Test>::get(signer), Some(true));

// 		assert_noop!(
// 			DefenseModule::safe_transfer(Origin::signed(signer), to, 10),
// 			Error::<Test>::AccountHasBeenFrozenTemporary
// 		);
// 	});
// }
