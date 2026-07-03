import { createFileRoute } from '@tanstack/react-router'
import { ProductList } from '../components/ProductList'
import { ProductForm } from '../components/ProductForm'

export const Route = createFileRoute('/products')({
  component: Products,
})

function Products() {
  return (
    <div className="p-2">
      <h2 className="text-xl font-bold mb-4">Products</h2>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="md:col-span-2">
          <ProductList />
        </div>
        <div>
          <ProductForm />
        </div>
      </div>
    </div>
  )
}
