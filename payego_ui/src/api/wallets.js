import client from './client';

export const walletApi = {
    getWallets: () => client.get('/api/wallets'),
};
