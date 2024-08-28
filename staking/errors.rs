use ink::prelude::string::String;

#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum StakingError {
    /// Custom error type for implementation-based errors.
    Custom(String),
    OverFlow,
    NotApproved,
    TransferFail,
    LowLiquidity

    
}
