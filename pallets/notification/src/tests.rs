// use super::*;
// use crate::{mock::*, Error};
// use frame_support::{assert_noop, assert_ok, pallet_prelude::*, traits::ConstU32, BoundedVec};
// use pallet_difttt::Action;

// #[test]
// fn test_set_mail_shold_work() {
// 	new_test_ext().execute_with(|| {
// 		assert_ok!(NotificationModule::set_mail(
// 			Origin::signed(0),
// 			vec![1, 2, 3],
// 			vec![4, 5, 6],
// 			vec![7, 8, 9]
// 		));

// 		let mut bound_receiver = BoundedVec::<u8, ConstU32<128>>::default();
// 		let mut bound_title = BoundedVec::<u8, ConstU32<128>>::default();
// 		let mut bound_body = BoundedVec::<u8, ConstU32<256>>::default();

// 		for x in vec![1, 2, 3].iter() {
// 			bound_receiver.try_push(*x);
// 		}
// 		for x in vec![4, 5, 6].iter() {
// 			bound_title.try_push(*x);
// 		}
// 		for x in vec![7, 8, 9].iter() {
// 			bound_body.try_push(*x);
// 		}

// 		let mail = Action::MailWithToken(
// 			Default::default(),
// 			Default::default(),
// 			bound_receiver,
// 			bound_title,
// 			bound_body,
// 		);

// 		assert_eq!(MapNofityAction::<Test>::get(0, 0), Some(mail));

// 		println!("@@@@@ {:?}", NotificationModule::get_mail_config_action(0));
// 	})
// }

// #[test]
// fn test_set_slack_shold_work() {
// 	new_test_ext().execute_with(|| {
// 		assert_ok!(NotificationModule::set_slack(Origin::signed(0), vec![1, 2, 3], vec![4, 5, 6],));

// 		let mut bound_hook_url = BoundedVec::<u8, ConstU32<256>>::default();
// 		let mut bound_message = BoundedVec::<u8, ConstU32<256>>::default();

// 		for x in vec![1, 2, 3].iter() {
// 			bound_hook_url.try_push(*x);
// 		}
// 		for x in vec![4, 5, 6].iter() {
// 			bound_message.try_push(*x);
// 		}

// 		let slack = Action::Slack(bound_hook_url, bound_message);
// 		assert_eq!(MapNofityAction::<Test>::get(0, 3), Some(slack));

// 		println!("@@@@@ {:?}", NotificationModule::get_slack_config_action(0));
// 	})
// }

// #[test]
// fn test_set_discord_shold_work() {
// 	new_test_ext().execute_with(|| {
// 		assert_ok!(NotificationModule::set_discord(
// 			Origin::signed(0),
// 			vec![1, 2, 3],
// 			vec![4, 5, 6],
// 			vec![7, 8, 9]
// 		));

// 		let mut bound_hook_url = BoundedVec::<u8, ConstU32<256>>::default();
// 		let mut bound_user = BoundedVec::<u8, ConstU32<64>>::default();
// 		let mut bound_content = BoundedVec::<u8, ConstU32<256>>::default();

// 		for x in vec![1, 2, 3].iter() {
// 			bound_hook_url.try_push(*x);
// 		}
// 		for x in vec![4, 5, 6].iter() {
// 			bound_user.try_push(*x);
// 		}
// 		for x in vec![7, 8, 9].iter() {
// 			bound_content.try_push(*x);
// 		}

// 		let mail = Action::Discord(bound_hook_url, bound_user, bound_content);
// 		assert_eq!(MapNofityAction::<Test>::get(0, 4), Some(mail));

// 		println!("@@@@@ {:?}", NotificationModule::get_discord_config_action(0));
// 	})
// }
