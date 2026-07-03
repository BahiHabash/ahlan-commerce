import { GraphQLClient } from 'graphql-request';

const API_URL = 'http://localhost:3000/graphql'; // Assuming API runs on port 3000

export const client = new GraphQLClient(API_URL, {
  // If we need any global headers (like auth), add them here
  headers: () => ({}),
});
