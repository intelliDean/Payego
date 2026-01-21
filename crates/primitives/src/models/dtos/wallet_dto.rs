use crate::models::enum_types::CurrencyCode;
use crate::models::wallet::Wallet;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletDto {
    pub id: Uuid,
    pub currency: CurrencyCode,
    pub balance: i64, // cents
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WalletsResponse {
    pub wallets: Vec<WalletDto>,
}

impl From<Wallet> for WalletDto {
    fn from(wallet: Wallet) -> Self {
        Self {
            id: wallet.id,
            currency: wallet.currency,
            balance: wallet.balance,
        }
    }
}
