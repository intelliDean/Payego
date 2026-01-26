import client from './client';
import {Wallet} from '@/types';

export const walletApi = {
    getWallets: () => client.get<{ wallets: Wallet[] }>('/api/user/wallets').then(res => res.data.wallets),
};
