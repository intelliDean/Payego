import client from './client';
import { Transaction } from '@/types';

export const transactionApi = {
    getTransactions: () => client.get<{ transactions: Transaction[] }>('/api/user/transactions').then(res => res.data.transactions),
    getTransactionDetails: (id: string) => client.get<Transaction>(`/api/transactions/${id}`).then(res => res.data),
    topUp: (data: any) => {
        // Generate a unique idempotency key for this request
        const idempotencyKey = crypto.randomUUID();
        return client.post('/api/wallet/top_up', {
            ...data,
            amount: Math.round(data.amount * 100),
            idempotency_key: idempotencyKey
        }).then(res => res.data);
    },
    internalTransfer: (data: any) => client.post('/api/transfer/internal', {
        ...data,
        amount: Math.round(data.amount * 100)
    }).then(res => res.data),
    externalTransfer: (data: any) => client.post('/api/transfer/external', {
        ...data,
        amount: Math.round(data.amount * 100)
    }).then(res => res.data),
    withdraw: (bankAccountId: string, data: any) => client.post(`/api/wallet/withdraw/${bankAccountId}`, {
        ...data,
        amount: Math.round(data.amount * 100)
    }).then(res => res.data),
    convertCurrency: (data: any) => client.post('/api/wallets/convert', {
        ...data,
        amount_cents: Math.round((data.amount_cents || data.amount) * 100)
    }).then(res => res.data),
    getExchangeRate: (from: string, to: string) => client.get<{ from: string, to: string, rate: number }>(`/api/exchange-rate?from=${from}&to=${to}`).then(res => res.data),
};
