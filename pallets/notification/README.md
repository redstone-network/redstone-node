# Permission Capture Module

Users can configure their own friend list in advance. When the user realizes that the private key has been stolen, he can contact his friend to vote for the account ownership. When the number of votes exceeds the threshold M, all permissions of the account will be managed by the friend. Any transfer can only be made after passing the vote. implement.

### Terminology

* **Is Public:** Public or private.
* **Backups:** Number of duplicates.
* **Deadline:** Expiration time.

## Interface

### Dispatchable Functions
* `set_mail` - Configure Email alert methods for users.
* `set_slack` - Configure Slack alert methods for users.
* `set_discord` - Configure Discord alert methods for users.
