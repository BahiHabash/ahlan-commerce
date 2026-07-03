import { useQuery } from '@tanstack/react-query'
import { client } from '../api/graphql'
import { gql } from 'graphql-request'

const LIST_PRODUCTS = gql`
  query ListProducts {
    products {
      id
      title
      handle
      priceCents
      inventoryQuantity
      published
      description
      createdAt
      updatedAt
      publishedAt
    }
  }
`

export type Product = {
  id: string
  title: string
  handle: string
  priceCents: number
  inventoryQuantity: number
  published: boolean
  description: string
  createdAt: string
  updatedAt: string
  publishedAt: string
}

export function useProducts() {
  return useQuery({
    queryKey: ['products'],
    queryFn: async () => {
      const data = await client.request<{ products: Product[] }>(LIST_PRODUCTS)
      return data.products
    },
  })
}
