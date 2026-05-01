CREATE SCHEMA IF NOT EXISTS test_schema;

CREATE TABLE test_schema.users (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP
);

CREATE TABLE test_schema.products (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    price NUMERIC(10, 2) NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 0,
    tags TEXT[],
    image_data BYTEA,
    product_uuid UUID NOT NULL DEFAULT gen_random_uuid(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE test_schema.orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES test_schema.users(id),
    product_id INTEGER NOT NULL REFERENCES test_schema.products(id),
    quantity INTEGER NOT NULL,
    total_price NUMERIC(10, 2) NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    notes TEXT,
    ordered_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_orders_user_id ON test_schema.orders(user_id);
CREATE INDEX idx_orders_product_id ON test_schema.orders(product_id);
CREATE INDEX idx_orders_status ON test_schema.orders(status);
CREATE INDEX idx_products_name ON test_schema.products(name);
CREATE UNIQUE INDEX idx_users_email ON test_schema.users(email);

CREATE VIEW test_schema.order_summary AS
SELECT
    o.id AS order_id,
    u.username,
    p.name AS product_name,
    o.quantity,
    o.total_price,
    o.status,
    o.ordered_at
FROM test_schema.orders o
JOIN test_schema.users u ON o.user_id = u.id
JOIN test_schema.products p ON o.product_id = p.id;

INSERT INTO test_schema.users (username, email, active, metadata) VALUES
('alice', 'alice@example.com', true, '{"role": "admin", "preferences": {"theme": "dark"}}'),
('bob', 'bob@example.com', true, '{"role": "user"}'),
('charlie', 'charlie@example.com', false, NULL),
('diana', 'diana@example.com', true, '{"role": "user", "preferences": {"theme": "light"}}'),
('eve', 'eve@example.com', true, '{"role": "moderator"}');

INSERT INTO test_schema.products (name, description, price, quantity, tags) VALUES
('Widget A', 'A standard widget', 9.99, 100, ARRAY['hardware', 'widget']),
('Widget B', 'A premium widget', 19.99, 50, ARRAY['hardware', 'widget', 'premium']),
('Gadget X', 'An electronic gadget', 49.99, 25, ARRAY['electronics', 'gadget']),
('Gadget Y', NULL, 99.99, 10, ARRAY['electronics']),
('Thingamajig', 'A mysterious thing', 4.99, 200, NULL);

INSERT INTO test_schema.orders (user_id, product_id, quantity, total_price, status) VALUES
(1, 1, 2, 19.98, 'completed'),
(1, 3, 1, 49.99, 'completed'),
(2, 2, 3, 59.97, 'pending'),
(2, 5, 10, 49.90, 'shipped'),
(3, 1, 1, 9.99, 'cancelled'),
(4, 4, 1, 99.99, 'pending'),
(4, 2, 2, 39.98, 'completed'),
(5, 3, 1, 49.99, 'shipped');

DO $$
BEGIN
    FOR i IN 6..100 LOOP
        INSERT INTO test_schema.users (username, email, active, metadata)
        VALUES (
            'user_' || i,
            'user_' || i || '@example.com',
            i % 5 != 0,
            json_build_object('role', CASE WHEN i % 10 = 0 THEN 'admin' ELSE 'user' END)::jsonb
        );
    END LOOP;
END $$;
