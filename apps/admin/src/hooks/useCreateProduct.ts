import { useMutation, useQueryClient } from '@tanstack/react-query'
import { client } from '../api/graphql'
import { gql } from 'graphql-request'
import { type Product } from './useProducts'

const CREATE_PRODUCT = gql`
  mutation CreateProduct($input: ProductCreateInput!) {
    productCreate(input: $input) {
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

export type ProductCreateInput = {
  title: string
  handle: string
  priceCents: number
  inventoryQuantity: number
  published: boolean
  description: string
}

export function useCreateProduct() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (input: ProductCreateInput) => {
      const data = await client.request<{ productCreate: Product }>(CREATE_PRODUCT, { input })
      return data.productCreate
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['products'] })
    },
  })
}
