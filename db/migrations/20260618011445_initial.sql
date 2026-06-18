-- Create "products" table
CREATE TABLE "public"."products" (
  "id" text NOT NULL,
  "title" text NOT NULL,
  "handle" text NOT NULL,
  "price_cents" integer NOT NULL,
  "inventory_quantity" integer NOT NULL,
  "published" boolean NOT NULL,
  "created_at" timestamptz NOT NULL,
  "updated_at" timestamptz NOT NULL,
  "description" text NULL,
  "published_at" timestamptz NULL,
  PRIMARY KEY ("id"),
  CONSTRAINT "products_handle_key" UNIQUE ("handle")
);
