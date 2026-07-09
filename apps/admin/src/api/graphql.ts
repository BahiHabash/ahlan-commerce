import { GraphQLClient } from 'graphql-request';

const baseUrl = import.meta.env.VITE_ADMIN_PUBLIC_API_URL || 'http://localhost:3000';
const API_URL = `${baseUrl.replace(/\/$/, '')}/graphql`;

export const client = new GraphQLClient(API_URL, {
  // If we need any global headers (like auth), add them here
  headers: () => ({}),
});
