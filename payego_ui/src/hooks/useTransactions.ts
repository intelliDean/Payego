import { useQuery } from '@tanstack/react-query';
import { transactionApi } from '../api/transactions';

export const useTransactions = () => {
    return useQuery({
        queryKey: ['transactions'],
        queryFn: transactionApi.getTransactions,
    });
};

export const useTransactionDetails = (id: string) => {
    return useQuery({
        queryKey: ['transaction', id],
        queryFn: () => transactionApi.getTransactionDetails(id),
        enabled: !!id,
    });
};
