import client from './client';
import { BankAccount } from '../types';

export interface Bank {
    code: string;
    name: string;
}

export const bankApi = {
    getBanks: () => client.get<{ banks: Bank[] }>('/api/banks/all').then(res => res.data.banks),
    getUserBankAccounts: () => client.get<{ bank_accounts: BankAccount[] }>('/api/user/banks').then(res => res.data.bank_accounts),
    addBankAccount: (data: { account_number: string, bank_code: string, bank_name: string }) =>
        client.post('/api/banks/add', data),
    deleteBankAccount: (id: string) => client.delete(`/api/banks/${id}`),
};
