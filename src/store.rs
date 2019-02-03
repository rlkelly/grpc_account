pub trait DataStore {
    pub fn create_account(&self, account: u32, req_id: u64) -> Result<u64, ()> {}
    pub fn get_account_balance(account: i64) -> Result<i64, ()> {}
    pub fn execute_transfers(transfers: &[TransferComponent], req_id: i64) -> Result<(), ()> {}
}
