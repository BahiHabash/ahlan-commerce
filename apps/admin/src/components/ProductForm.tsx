import { useState } from 'react'
import { useCreateProduct, ProductCreateInput } from '../hooks/useCreateProduct'

export function ProductForm() {
  const { mutate, isPending, isError, error, isSuccess } = useCreateProduct()
  const [formData, setFormData] = useState<ProductCreateInput>({
    title: '',
    handle: '',
    priceCents: 0,
    inventoryQuantity: 0,
    published: false,
    description: '',
  })

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    const { name, value, type } = e.target
    const parsedValue = type === 'checkbox' ? (e.target as HTMLInputElement).checked
      : type === 'number' ? Number(value)
      : value
    setFormData(prev => ({ ...prev, [name]: parsedValue }))
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    mutate(formData, {
      onSuccess: () => {
        setFormData({
          title: '',
          handle: '',
          priceCents: 0,
          inventoryQuantity: 0,
          published: false,
          description: '',
        })
      }
    })
  }

  return (
    <div className="card">
      <h3 className="card-title">Create Product</h3>
      <form onSubmit={handleSubmit} className="modern-form">
        <div className="form-group">
          <label htmlFor="title">Title</label>
          <input id="title" name="title" required value={formData.title} onChange={handleChange} placeholder="Product Title" />
        </div>
        
        <div className="form-group">
          <label htmlFor="handle">Handle</label>
          <input id="handle" name="handle" required value={formData.handle} onChange={handleChange} placeholder="product-handle" />
        </div>

        <div className="form-row">
          <div className="form-group flex-1">
            <label htmlFor="priceCents">Price (cents)</label>
            <input id="priceCents" name="priceCents" type="number" required min="0" value={formData.priceCents} onChange={handleChange} />
          </div>
          <div className="form-group flex-1">
            <label htmlFor="inventoryQuantity">Inventory</label>
            <input id="inventoryQuantity" name="inventoryQuantity" type="number" required min="0" value={formData.inventoryQuantity} onChange={handleChange} />
          </div>
        </div>

        <div className="form-group">
          <label htmlFor="description">Description</label>
          <textarea id="description" name="description" rows={3} value={formData.description} onChange={handleChange} placeholder="Brief description..."></textarea>
        </div>

        <div className="form-group checkbox-group">
          <label className="checkbox-label">
            <input name="published" type="checkbox" checked={formData.published} onChange={handleChange} />
            <span className="slider"></span>
            Published
          </label>
        </div>

        {isError && <div className="error-message">Error: {error.message}</div>}
        {isSuccess && <div className="success-message">Product created successfully!</div>}

        <button type="submit" disabled={isPending} className="btn-primary">
          {isPending ? 'Creating...' : 'Create Product'}
        </button>
      </form>
    </div>
  )
}
