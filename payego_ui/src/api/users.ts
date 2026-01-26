import client from './client';
import { ResolvedUser } from '@/types';

export const usersApi = {
    resolveUser: (identifier: string) =>
        client.get<ResolvedUser>('/api/users/resolve', { params: { identifier } })
            .then(res => res.data),
};
