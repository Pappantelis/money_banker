-- Add migration script here
CREATE TABLE test_expenses (
    id SERIAL PRIMARY KEY,
    description TEXT NOT NULL,
    amount DECIMAL(10, 2) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Βάλε και μερικά test data
INSERT INTO test_expenses (description, amount) VALUES
    ('Καφές Starbucks', 4.50),
    ('Σούπερ Μάρκετ AB', 45.30),
    ('Βενζίνη Shell', 60.00);