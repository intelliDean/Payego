export interface User {
    id: string;
    username: string | null;
    email: string;
    created_at: string;
}

export type Currency = 'USD' | 'EUR' | 'GBP' | 'NGN' | 'CAD' | 'AUD' | 'CHF' | 'JPY' | 'CNY' | 'SEK' | 'NZD' | 'MXN' | 'SGD' | 'HKD' | 'NOK' | 'KRW' | 'TRY' | 'INR' | 'BRL' | 'ZAR';

export interface Wallet {
    id: string;
    user_id: string;
    currency: Currency;
    balance: number;
    created_at: string;
    updated_at: string;
}

export interface BankAccount {
    id: string;
    user_id: string;
    bank_name: string;
    account_number: string;
    account_holder_name: string;
    currency: Currency;
    created_at: string;
}

export type TransactionType = 'TopUp' | 'Payout' | 'Transfer' | 'Conversion' | 'Withdrawal' | 'InternalTransfer' | 'ExternalTransfer' | 'CurrencyConversion';
export type TransactionStatus = 'Pending' | 'Completed' | 'Failed' | 'Reversed';

export interface Transaction {
    id: string;
    user_id: string;
    wallet_id?: string;
    intent: TransactionType;
    amount: number;
    currency: Currency;
    status: TransactionStatus;
    reference: string;
    metadata?: Record<string, any>;
    created_at: string;
    updated_at: string;
}

export interface ApiResponse<T> {
    data: T;
    message?: string;
}

export interface ApiError {
    error: string;
    message: string;
    code?: string;
}

export interface ResolvedUser {
    id: string;
    email: string;
    username: string | null;
}
