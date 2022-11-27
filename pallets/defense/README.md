# Defense Module

Provide passive defense functions, users configure different transaction limits and transaction frequencies, alarm conditions and methods for addresses, and reduce asset losses in emergency situations through the freezing function.

### Terminology

* **Is Public:** Public or private.
* **Backups:** Number of duplicates.
* **Deadline:** Expiration time.

## Interface

### Dispatchable Functions
* `set_transfer_limit` - The transaction limit and transaction frequency of the user-configured address.
* `set_risk_management` - Freezing time for user-configured addresses.
* `safe_transfer` - Transfer method, check whether the transfer is safe according to user configuration.
* `freeze_account` - Trigger the freezing operation, after the user clicks, the account cannot be transferred within the specified time.
* `reset_notification_status` - xxx.
