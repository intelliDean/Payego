import { useQuery } from '@tanstack/react-query';
import { walletApi } from '../api/wallets';

export const useWallets = () => {
    return useQuery({
        queryKey: ['wallets'],
        queryFn: walletApi.getWallets,
    });
};
