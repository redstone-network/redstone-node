pub trait NotificationInfoInterface<AccountId, Action> {
	fn get_mail_config_action(account: AccountId) -> Option<Action>;
	fn get_slack_config_action(account: AccountId) -> Option<Action>;
	fn get_discord_config_action(account: AccountId) -> Option<Action>;
}
