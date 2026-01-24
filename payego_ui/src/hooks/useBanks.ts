import { useQuery } from '@tanstack/react-query';
import { bankApi } from '../api/bank';

export const useBanks = () => {
    return useQuery({
        queryKey: ['banks'],
        queryFn: bankApi.getBanks,
    });
};

export const useUserBankAccounts = () => {
    return useQuery({
        queryKey: ['user-banks'],
        queryFn: bankApi.getUserBankAccounts,
    });
};
