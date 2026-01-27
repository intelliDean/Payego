import client from './client';
import {BankAccount} from '@/types';

export interface Bank {
    code: string;
    name: string;
}

export const bankApi = {
    getBanks: () => client.get<{ banks: Bank[] }>('/api/banks/all').then(res => res.data.banks),
    getUserBankAccounts: () => client.get<{
        bank_accounts: BankAccount[]
    }>('/api/user/banks').then(res => res.data.bank_accounts),
    addBankAccount: (data: { account_number: string, bank_code: string, bank_name: string }) =>
        client.post('/api/banks/add', data),
    resolveAccount: (bankCode: string, accountNumber: string) =>
        client.get<{
            account_name: string
        }>(`/api/bank/resolve?account_number=${accountNumber}&bank_code=${bankCode}`).then(res => res.data),
    deleteBankAccount: (id: string) => client.delete(`/api/banks/${id}`),
};
