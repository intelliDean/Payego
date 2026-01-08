import client from './client';

export const transactionApi = {
    getTransactions: () => client.get('/api/transactions'),
    getTransactionDetails: (id) => client.get(`/api/transactions/${id}`),
    topUp: (data) => client.post('/api/top_up', data),
    internalTransfer: (data) => client.post('/api/transfer/internal', data),
    externalTransfer: (data) => client.post('/api/transfer/external', data),
    withdraw: (data) => client.post('/api/withdraw', data),
    convertCurrency: (data) => client.post('/api/convert_currency', data),
};
