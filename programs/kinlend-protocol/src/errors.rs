use anchor_lang::error_code;


#[error_code]
pub enum ErrorCode {
    #[msg("Loan is already funded.")]
    AlreadyFunded,

    #[msg("Loan Request is not funded yet.")]
    NotFunded,

    #[msg("Loan has expired.")]
    LoanExpired,

    #[msg("Loan is not expired.")]
    LoanIsNotExpired,

    #[msg("Loan Request not found in the registry.")]
    NotFoundInRegistry,

    #[msg("Calculation Error.")]
    CalculationError,

    #[msg("Page is full.")]
    PageIsFull,

    #[msg("Insufficient Collateral")]
    InsuffientCollateral
}