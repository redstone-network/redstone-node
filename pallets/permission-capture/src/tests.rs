use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_create_get_account_permissions() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PermissionCaptureModule::create_get_account_permissions(Origin::signed(1), 0),
			Error::<Test>::NotCaptureable
		);

		assert_ok!(PermissionCaptureModule::create_capture_config(
			Origin::signed(0),
			vec![1, 2, 3],
			2
		));

		assert_noop!(
			PermissionCaptureModule::create_get_account_permissions(Origin::signed(4), 0),
			Error::<Test>::MustProposalByFriends
		);
		assert_ok!(PermissionCaptureModule::create_get_account_permissions(Origin::signed(1), 0));
		assert_noop!(
			PermissionCaptureModule::create_get_account_permissions(Origin::signed(1), 0),
			Error::<Test>::ProposalAlreadyCreated
		);

		assert_eq!(
			ActiveCaptures::<Test>::get(0),
			Some(ActiveCapture {
				created: <frame_system::Pallet<Test>>::block_number(),
				friends_approve: Default::default(),
				friends_cancel: Default::default(),

				execute_proposal_id: 0,
				cancel_proposal_id: Default::default(),
				capture_statue: CaptureStatue::Processing,
			})
		);

		assert_eq!(
			Proposals::<Test>::get(0),
			Some(Proposal {
				proposal_type: ProposalType::ExecuteCapture,
				proposer: 1,
				capture_owner: 0,

				approve_votes: Default::default(),
				deny_votes: Default::default(),

				statue: ProposalStatue::Processing,
			})
		);
	});
}

#[test]
fn it_works_for_cancel_get_account_permissions() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PermissionCaptureModule::cancel_get_account_permissions(Origin::signed(1), 0),
			Error::<Test>::NotCaptureable
		);

		assert_ok!(PermissionCaptureModule::create_capture_config(
			Origin::signed(0),
			vec![1, 2, 3],
			2
		));

		assert_noop!(
			PermissionCaptureModule::cancel_get_account_permissions(Origin::signed(1), 0),
			Error::<Test>::NotStarted
		);

		assert_ok!(PermissionCaptureModule::create_get_account_permissions(Origin::signed(1), 0));

		assert_noop!(
			PermissionCaptureModule::cancel_get_account_permissions(Origin::signed(5), 0),
			Error::<Test>::MustProposalByFriends
		);

		assert_ok!(PermissionCaptureModule::cancel_get_account_permissions(Origin::signed(1), 0));

		assert_eq!(
			ActiveCaptures::<Test>::get(0),
			Some(ActiveCapture {
				created: <frame_system::Pallet<Test>>::block_number(),
				friends_approve: Default::default(),
				friends_cancel: Default::default(),

				execute_proposal_id: 0,
				cancel_proposal_id: Some(1),
				capture_statue: CaptureStatue::Processing,
			})
		);

		assert_eq!(
			Proposals::<Test>::get(1),
			Some(Proposal {
				proposal_type: ProposalType::Cancel,
				proposer: 1,
				capture_owner: 0,

				approve_votes: Default::default(),
				deny_votes: Default::default(),

				statue: ProposalStatue::Processing,
			})
		);
	});
}

#[test]
fn it_works_for_vote_pass() {
	new_test_ext().execute_with(|| {
		assert_ok!(PermissionCaptureModule::create_capture_config(
			Origin::signed(0),
			vec![1, 2, 3],
			2
		));

		assert_ok!(PermissionCaptureModule::create_get_account_permissions(Origin::signed(1), 0));

		assert_noop!(
			PermissionCaptureModule::vote(Origin::signed(5), 0, 0),
			Error::<Test>::MustVoteByFriends
		);

		assert_eq!(MapPermissionTaken::<Test>::get(0), None);

		// first vote
		assert_ok!(PermissionCaptureModule::vote(Origin::signed(1), 0, 0));

		assert_eq!(
			Proposals::<Test>::get(0),
			Some(Proposal {
				proposal_type: ProposalType::ExecuteCapture,
				proposer: 1,
				capture_owner: 0,
				approve_votes: 1,
				deny_votes: Default::default(),
				statue: ProposalStatue::Processing,
			})
		);

		let bounded_friends: FriendsOf<Test> =
			match vec![1].clone().try_into().map_err(|()| Error::<Test>::MaxFriends) {
				Ok(v) => v,
				_ => Default::default(),
			};
		assert_eq!(
			ActiveCaptures::<Test>::get(0),
			Some(ActiveCapture {
				created: <frame_system::Pallet<Test>>::block_number(),
				friends_approve: bounded_friends,
				friends_cancel: Default::default(),
				execute_proposal_id: 0,
				cancel_proposal_id: Default::default(),
				capture_statue: CaptureStatue::Processing,
			})
		);

		assert_noop!(
			PermissionCaptureModule::vote(Origin::signed(1), 0, 0),
			Error::<Test>::AlreadyApproved
		);

		// second votes
		assert_ok!(PermissionCaptureModule::vote(Origin::signed(2), 0, 0));

		assert_eq!(
			Proposals::<Test>::get(0),
			Some(Proposal {
				proposal_type: ProposalType::ExecuteCapture,
				proposer: 1,
				capture_owner: 0,
				approve_votes: 2,
				deny_votes: Default::default(),
				statue: ProposalStatue::End,
			})
		);

		let bounded_friends: FriendsOf<Test> =
			match vec![1, 2].clone().try_into().map_err(|()| Error::<Test>::MaxFriends) {
				Ok(v) => v,
				_ => Default::default(),
			};
		assert_eq!(
			ActiveCaptures::<Test>::get(0),
			Some(ActiveCapture {
				created: <frame_system::Pallet<Test>>::block_number(),
				friends_approve: bounded_friends,
				friends_cancel: Default::default(),
				execute_proposal_id: 0,
				cancel_proposal_id: Default::default(),
				capture_statue: CaptureStatue::PermissionTaken,
			})
		);

		assert_eq!(MapPermissionTaken::<Test>::get(0), Some(()));

		// third votes
		assert_noop!(
			PermissionCaptureModule::vote(Origin::signed(3), 0, 0),
			Error::<Test>::ProposalMustProcessing
		);
	});
}

#[test]
fn it_works_for_vote_cancel() {
	new_test_ext().execute_with(|| {
		assert_ok!(PermissionCaptureModule::create_capture_config(
			Origin::signed(0),
			vec![1, 2, 3],
			2
		));

		assert_ok!(PermissionCaptureModule::create_get_account_permissions(Origin::signed(1), 0));
		assert_ok!(PermissionCaptureModule::cancel_get_account_permissions(Origin::signed(1), 0));

		// first vote
		assert_ok!(PermissionCaptureModule::vote(Origin::signed(1), 1, 0));

		assert_eq!(
			Proposals::<Test>::get(0),
			Some(Proposal {
				proposal_type: ProposalType::ExecuteCapture,
				proposer: 1,
				capture_owner: 0,
				approve_votes: 0,
				deny_votes: Default::default(),
				statue: ProposalStatue::Processing,
			})
		);

		assert_eq!(
			Proposals::<Test>::get(1),
			Some(Proposal {
				proposal_type: ProposalType::Cancel,
				proposer: 1,
				capture_owner: 0,
				approve_votes: 1,
				deny_votes: Default::default(),
				statue: ProposalStatue::Processing,
			})
		);

		let bounded_friends: FriendsOf<Test> =
			match vec![1].clone().try_into().map_err(|()| Error::<Test>::MaxFriends) {
				Ok(v) => v,
				_ => Default::default(),
			};
		assert_eq!(
			ActiveCaptures::<Test>::get(0),
			Some(ActiveCapture {
				created: <frame_system::Pallet<Test>>::block_number(),
				friends_approve: Default::default(),
				friends_cancel: bounded_friends,
				execute_proposal_id: 0,
				cancel_proposal_id: Some(1),
				capture_statue: CaptureStatue::Processing,
			})
		);

		// second votes
		assert_ok!(PermissionCaptureModule::vote(Origin::signed(2), 1, 0));

		assert_eq!(
			Proposals::<Test>::get(0),
			Some(Proposal {
				proposal_type: ProposalType::ExecuteCapture,
				proposer: 1,
				capture_owner: 0,
				approve_votes: 0,
				deny_votes: Default::default(),
				statue: ProposalStatue::End,
			})
		);
		assert_eq!(
			Proposals::<Test>::get(1),
			Some(Proposal {
				proposal_type: ProposalType::Cancel,
				proposer: 1,
				capture_owner: 0,
				approve_votes: 2,
				deny_votes: Default::default(),
				statue: ProposalStatue::End,
			})
		);

		let bounded_friends: FriendsOf<Test> =
			match vec![1, 2].clone().try_into().map_err(|()| Error::<Test>::MaxFriends) {
				Ok(v) => v,
				_ => Default::default(),
			};
		assert_eq!(
			ActiveCaptures::<Test>::get(0),
			Some(ActiveCapture {
				created: <frame_system::Pallet<Test>>::block_number(),
				friends_approve: Default::default(),
				friends_cancel: bounded_friends,
				execute_proposal_id: 0,
				cancel_proposal_id: Some(1),
				capture_statue: CaptureStatue::Canceled,
			})
		);

		// third votes
		assert_noop!(
			PermissionCaptureModule::vote(Origin::signed(3), 1, 0),
			Error::<Test>::ProposalMustProcessing
		);
	});
}

#[test]
fn it_works_for_create_capture_config() {
	new_test_ext().execute_with(|| {
		assert_ok!(PermissionCaptureModule::create_capture_config(
			Origin::signed(0),
			vec![1, 2, 3],
			2
		));

		let bounded_friends: FriendsOf<Test> =
			match vec![1, 2, 3].clone().try_into().map_err(|()| Error::<Test>::MaxFriends) {
				Ok(v) => v,
				_ => Default::default(),
			};
		assert_eq!(
			Captureable::<Test>::get(0),
			Some(CaptureConfig { friends: bounded_friends, threshold: 2 })
		);
	});
}

// #[test]
// fn it_works_for_operational_voting() {
// 	new_test_ext().execute_with(|| {

// 	});
// }
