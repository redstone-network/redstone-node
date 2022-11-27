# Permission Capture Module

Users can configure their own friend list in advance. When the user realizes that the private key has been stolen, he can contact his friend to vote for the account ownership. When the number of votes exceeds the threshold M, all permissions of the account will be managed by the friend. Any transfer can only be made after passing the vote. implement.

### Terminology

* **Is Public:** Public or private.
* **Backups:** Number of duplicates.
* **Deadline:** Expiration time.

## Interface

### Dispatchable Functions
* `create_get_account_permissions` - A friend initiates a proposal, and the ownership of the application address is transferred to the status of friend management.
* `cancel_get_account_permissions` - A friend cancels a proposal initiated by himself.
* `vote` - When the account is in takeover state, friends can use this method to vote on the address transaction application.
* `create_capture_config` - The user creates a power grab configuration and writes his friend list into this method.
* `operational_voting` - xxx.
