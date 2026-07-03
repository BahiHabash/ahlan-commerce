import { useProducts } from '../hooks/useProducts'

export function ProductList() {
  const { data: products, isLoading, isError, error } = useProducts()

  if (isLoading) return <div className="loader">Loading products...</div>
  if (isError) return <div className="error">Error loading products: {error.message}</div>
  if (!products || products.length === 0) return <div className="empty">No products found.</div>

  return (
    <div className="card">
      <h3 className="card-title">All Products</h3>
      <div className="table-container">
        <table className="modern-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Handle</th>
              <th>Price</th>
              <th>Inventory</th>
              <th>Status</th>
              <th>Created At</th>
            </tr>
          </thead>
          <tbody>
            {products.map((p) => (
              <tr key={p.id}>
                <td><strong>{p.title}</strong></td>
                <td><span className="badge handle">{p.handle}</span></td>
                <td>{(p.priceCents / 100).toLocaleString('en-US', { style: 'currency', currency: 'USD' })}</td>
                <td>{p.inventoryQuantity}</td>
                <td>
                  <span className={`badge ${p.published ? 'published' : 'draft'}`}>
                    {p.published ? 'Published' : 'Draft'}
                  </span>
                </td>
                <td>{new Date(p.createdAt).toLocaleDateString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}
