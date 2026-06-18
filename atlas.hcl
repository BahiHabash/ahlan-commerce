env "local" {
  src = "file://db/schema/products.sql"
  url = "postgres://postgres@localhost:5432/ahlan_commerce?sslmode=disable"
  dev = "postgres://postgres@localhost:5432/ahlan_commerce_dev?sslmode=disable"
  migration {
    dir = "file://db/migrations"
  }
  format {
    migrate {
      diff = "{{ sql . \"  \" }}"
    }
  }
}
